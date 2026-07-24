use std::cell::{Cell, RefCell};
use std::env;
use std::io;
use std::rc::Rc;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gio, glib};
use vte::prelude::*;

use super::{CloseRequest, ProcessExit, TerminalError, TerminalEvent};

const SPAWN_TIMEOUT_MILLISECONDS: i32 = 10_000;
const GRACEFUL_CLOSE_DELAY: Duration = Duration::from_millis(1_500);
const FORCED_CLOSE_DELAY: Duration = Duration::from_millis(2_500);
const PCRE2_MULTILINE: u32 = 0x0000_0400;

type EventHandler = Rc<dyn Fn(TerminalEvent)>;
type CloseCallback = Box<dyn FnOnce()>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProcessState {
    Idle,
    Starting,
    Running(libc::pid_t),
    Closing(Option<libc::pid_t>),
    Exited,
    Failed,
}

#[derive(Clone)]
pub(super) struct VteBackend {
    terminal: vte::Terminal,
    state: Rc<Cell<ProcessState>>,
    cancellable: Rc<RefCell<Option<gio::Cancellable>>>,
    event_handler: Rc<RefCell<Option<EventHandler>>>,
    close_callbacks: Rc<RefCell<Vec<CloseCallback>>>,
    search_regex: Rc<RefCell<Option<vte::Regex>>>,
}

impl VteBackend {
    pub(super) fn new() -> Self {
        let terminal = vte::Terminal::new();
        terminal.set_hexpand(true);
        terminal.set_vexpand(true);

        // Enable OSC 8 hyperlinks.
        terminal.set_allow_hyperlink(true);

        let backend = Self {
            terminal,
            state: Rc::new(Cell::new(ProcessState::Idle)),
            cancellable: Rc::new(RefCell::new(None)),
            event_handler: Rc::new(RefCell::new(None)),
            close_callbacks: Rc::new(RefCell::new(Vec::new())),
            search_regex: Rc::new(RefCell::new(None)),
        };
        backend.setup_url_matching();
        backend.connect_child_exit();
        backend
    }

    /// Register a URL-matching regex so plain URLs get highlighted.
    fn setup_url_matching(&self) {
        // Match http:// and https:// URLs.
        let url_pattern = r"https?://[-a-zA-Z0-9+&@#/%?=~_|!:,.;]+[-a-zA-Z0-9+&@#/%=~_|]";
        let Ok(regex) = vte::Regex::for_match(url_pattern, PCRE2_MULTILINE) else {
            return;
        };
        let tag = self.terminal.match_add_regex(&regex, 0);
        self.terminal.match_set_cursor_name(tag, "pointer");
    }

    pub(super) fn widget(&self) -> gtk::Widget {
        self.terminal.clone().upcast()
    }

    pub(super) fn set_event_handler(&self, handler: EventHandler) {
        *self.event_handler.borrow_mut() = Some(handler);
    }

    pub(super) fn spawn(&self, program: &str, args: &[String], working_directory: &str) {
        if self.state.replace(ProcessState::Starting) != ProcessState::Idle {
            return;
        }

        let cancellable = gio::Cancellable::new();
        *self.cancellable.borrow_mut() = Some(cancellable.clone());

        // Build argv: program first, then any additional arguments.
        let mut argv: Vec<&str> = Vec::with_capacity(1 + args.len());
        argv.push(program);
        for argument in args {
            argv.push(argument.as_str());
        }
        let environment = inherited_environment();
        let environment: Vec<&str> = environment.iter().map(String::as_str).collect();
        let state = self.state.clone();
        let stored_cancellable = self.cancellable.clone();
        let event_handler = self.event_handler.clone();
        let close_callbacks = self.close_callbacks.clone();

        self.terminal.spawn_async(
            vte::PtyFlags::DEFAULT,
            Some(working_directory),
            &argv,
            &environment,
            glib::SpawnFlags::DEFAULT,
            || {},
            SPAWN_TIMEOUT_MILLISECONDS,
            Some(&cancellable),
            move |result| {
                stored_cancellable.borrow_mut().take();

                match result {
                    Ok(pid) => match state.get() {
                        ProcessState::Starting => {
                            state.set(ProcessState::Running(pid.0));
                            emit_event(&event_handler, TerminalEvent::Started);
                        }
                        ProcessState::Closing(None) => {
                            state.set(ProcessState::Closing(Some(pid.0)));
                            let _ = send_signal(pid.0, libc::SIGHUP);
                        }
                        _ => {}
                    },
                    Err(error) => match state.get() {
                        ProcessState::Closing(_) => {
                            state.set(ProcessState::Failed);
                            run_close_callbacks(&close_callbacks);
                        }
                        ProcessState::Starting => {
                            state.set(ProcessState::Failed);
                            emit_event(
                                &event_handler,
                                TerminalEvent::SpawnFailed(TerminalError::Spawn(error)),
                            );
                        }
                        _ => {}
                    },
                }
            },
        );
    }

    pub(super) fn copy(&self) -> Result<(), TerminalError> {
        if gtk::gdk::Display::default().is_none() {
            return Err(TerminalError::ClipboardUnavailable);
        }
        if !self.terminal.has_selection() {
            return Err(TerminalError::NoSelection);
        }

        self.terminal.copy_clipboard_format(vte::Format::Text);
        Ok(())
    }

    pub(super) fn paste(&self) -> Result<(), TerminalError> {
        if gtk::gdk::Display::default().is_none() {
            return Err(TerminalError::ClipboardUnavailable);
        }

        self.terminal.paste_clipboard();
        Ok(())
    }

    pub(super) fn focus(&self) {
        self.terminal.grab_focus();
    }

    pub(super) fn connect_title_changed<F>(&self, handler: F)
    where
        F: Fn(&str) + 'static,
    {
        self.terminal.connect_window_title_changed(move |term| {
            if let Some(title) = term.window_title() {
                handler(title.as_str());
            }
        });
    }

    pub(super) fn request_close<F>(&self, on_ready: F) -> CloseRequest
    where
        F: FnOnce() + 'static,
    {
        match self.state.get() {
            ProcessState::Idle | ProcessState::Exited | ProcessState::Failed => CloseRequest::Ready,
            ProcessState::Closing(_) => {
                self.close_callbacks.borrow_mut().push(Box::new(on_ready));
                CloseRequest::Pending
            }
            ProcessState::Starting => {
                self.close_callbacks.borrow_mut().push(Box::new(on_ready));
                self.state.set(ProcessState::Closing(None));
                if let Some(cancellable) = self.cancellable.borrow().as_ref() {
                    cancellable.cancel();
                }
                self.schedule_close_escalation();
                CloseRequest::Pending
            }
            ProcessState::Running(pid) => {
                self.close_callbacks.borrow_mut().push(Box::new(on_ready));
                self.state.set(ProcessState::Closing(Some(pid)));
                let _ = send_signal(pid, libc::SIGHUP);
                self.schedule_close_escalation();
                CloseRequest::Pending
            }
        }
    }

    pub(super) fn search(&self, query: &str) -> Result<(), TerminalError> {
        if query.is_empty() {
            self.search_clear();
            return Ok(());
        }

        let escaped = glib::Regex::escape_string(query);
        let regex = vte::Regex::for_match(&escaped, PCRE2_MULTILINE)
            .map_err(|_err| TerminalError::InvalidSearchPattern)?;

        self.terminal.search_set_regex(Some(&regex), 0);
        self.terminal.search_set_wrap_around(true);
        *self.search_regex.borrow_mut() = Some(regex);

        // Jump to first match.
        self.terminal.search_find_next();

        Ok(())
    }

    pub(super) fn search_next(&self) -> bool {
        self.terminal.search_find_next()
    }

    pub(super) fn search_previous(&self) -> bool {
        self.terminal.search_find_previous()
    }

    pub(super) fn search_clear(&self) {
        self.terminal.search_set_regex(None, 0);
        *self.search_regex.borrow_mut() = None;
    }

    pub(super) fn hyperlink_at(&self, x: f64, y: f64) -> Option<String> {
        let uri = self
            .terminal
            .check_hyperlink_at(x, y)
            .map(|value| value.to_string())
            .or_else(|| {
                self.terminal
                    .check_match_at(x, y)
                    .0
                    .map(|value| value.to_string())
            })?;

        allowed_http_uri(&uri)
    }

    pub(super) fn set_font(&self, font_desc: &gtk::pango::FontDescription) {
        self.terminal.set_font_desc(Some(font_desc));
    }

    pub(super) fn zoom_font(&self, delta: f64) {
        let scale = self.terminal.font_scale() + delta;
        let clamped = scale.clamp(0.5, 4.0);
        self.terminal.set_font_scale(clamped);
    }

    pub(super) fn reset_font_zoom(&self) {
        self.terminal.set_font_scale(1.0);
    }

    fn connect_child_exit(&self) {
        let state = self.state.clone();
        let event_handler = self.event_handler.clone();
        let close_callbacks = self.close_callbacks.clone();

        self.terminal.connect_child_exited(move |terminal, status| {
            terminal.set_input_enabled(false);

            match state.get() {
                ProcessState::Closing(_) => {
                    state.set(ProcessState::Exited);
                    run_close_callbacks(&close_callbacks);
                }
                ProcessState::Exited | ProcessState::Failed => {}
                _ => {
                    state.set(ProcessState::Exited);
                    emit_event(
                        &event_handler,
                        TerminalEvent::Exited(ProcessExit::from_wait_status(status)),
                    );
                }
            }
        });
    }

    fn schedule_close_escalation(&self) {
        let state = self.state.clone();
        glib::timeout_add_local_once(GRACEFUL_CLOSE_DELAY, move || {
            if let ProcessState::Closing(Some(pid)) = state.get() {
                let _ = send_signal(pid, libc::SIGKILL);
            }
        });

        let state = self.state.clone();
        let close_callbacks = self.close_callbacks.clone();
        glib::timeout_add_local_once(FORCED_CLOSE_DELAY, move || {
            if matches!(state.get(), ProcessState::Closing(_)) {
                state.set(ProcessState::Exited);
                run_close_callbacks(&close_callbacks);
            }
        });
    }
}

fn emit_event(handler: &RefCell<Option<EventHandler>>, event: TerminalEvent) {
    if let Some(handler) = handler.borrow().as_ref() {
        handler(event);
    }
}

fn run_close_callbacks(callbacks: &RefCell<Vec<CloseCallback>>) {
    let callbacks = std::mem::take(&mut *callbacks.borrow_mut());
    for callback in callbacks {
        callback();
    }
}

fn allowed_http_uri(uri: &str) -> Option<String> {
    let parsed = glib::Uri::parse(uri, glib::UriFlags::NONE).ok()?;
    let scheme = parsed.scheme();
    if !matches!(scheme.as_str(), "http" | "https") {
        return None;
    }
    if parsed.host().is_none_or(|host| host.is_empty()) {
        return None;
    }

    Some(uri.to_owned())
}

fn inherited_environment() -> Vec<String> {
    let mut environment: Vec<String> = env::vars_os()
        .filter_map(|(name, value)| {
            let name = name.into_string().ok()?;
            let value = value.into_string().ok()?;
            (!matches!(name.as_str(), "TERM" | "COLORTERM")).then(|| format!("{name}={value}"))
        })
        .collect();
    environment.push("TERM=xterm-256color".to_owned());
    environment.push("COLORTERM=truecolor".to_owned());
    environment
}

fn send_signal(pid: libc::pid_t, signal: libc::c_int) -> io::Result<()> {
    // SAFETY: `kill` is called with the child PID returned by VTE and a valid POSIX signal.
    let result = unsafe { libc::kill(pid, signal) };
    if result == 0 {
        return Ok(());
    }

    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::{CloseCallback, allowed_http_uri, inherited_environment, run_close_callbacks};

    #[test]
    fn terminal_capabilities_override_parent_values() {
        let environment = inherited_environment();
        let terminal_values: Vec<&str> = environment
            .iter()
            .filter(|entry| entry.starts_with("TERM="))
            .map(String::as_str)
            .collect();
        let color_terminal_values: Vec<&str> = environment
            .iter()
            .filter(|entry| entry.starts_with("COLORTERM="))
            .map(String::as_str)
            .collect();

        assert_eq!(terminal_values, ["TERM=xterm-256color"]);
        assert_eq!(color_terminal_values, ["COLORTERM=truecolor"]);
    }

    #[test]
    fn allows_http_and_https_links() {
        assert_eq!(
            allowed_http_uri("https://example.com/path?q=1"),
            Some("https://example.com/path?q=1".to_owned())
        );
        assert_eq!(
            allowed_http_uri("http://example.com"),
            Some("http://example.com".to_owned())
        );
    }

    #[test]
    fn rejects_unsafe_or_incomplete_links() {
        assert_eq!(allowed_http_uri("file:///etc/passwd"), None);
        assert_eq!(allowed_http_uri("javascript:alert(1)"), None);
        assert_eq!(allowed_http_uri("https:relative"), None);
        assert_eq!(allowed_http_uri("not a uri"), None);
    }

    #[test]
    fn close_callbacks_are_all_drained_exactly_once() {
        let calls = Rc::new(Cell::new(0));
        let callbacks: RefCell<Vec<CloseCallback>> = RefCell::new(
            (0..3)
                .map(|_| {
                    let calls = Rc::clone(&calls);
                    Box::new(move || calls.set(calls.get() + 1)) as CloseCallback
                })
                .collect(),
        );

        run_close_callbacks(&callbacks);
        run_close_callbacks(&callbacks);

        assert_eq!(calls.get(), 3);
        assert!(callbacks.borrow().is_empty());
    }
}
