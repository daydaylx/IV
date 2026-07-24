use std::cell::{Cell, RefCell};
use std::env;
use std::io;
use std::rc::Rc;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gio, glib};
use vte::prelude::*;

use super::{ProcessExit, TerminalError, TerminalEvent};

const SPAWN_TIMEOUT_MILLISECONDS: i32 = 10_000;
const GRACEFUL_CLOSE_DELAY: Duration = Duration::from_millis(1_500);
const FORCED_CLOSE_DELAY: Duration = Duration::from_millis(2_500);

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
    close_callback: Rc<RefCell<Option<CloseCallback>>>,
}

impl VteBackend {
    pub(super) fn new() -> Self {
        let terminal = vte::Terminal::new();
        terminal.set_hexpand(true);
        terminal.set_vexpand(true);

        let backend = Self {
            terminal,
            state: Rc::new(Cell::new(ProcessState::Idle)),
            cancellable: Rc::new(RefCell::new(None)),
            event_handler: Rc::new(RefCell::new(None)),
            close_callback: Rc::new(RefCell::new(None)),
        };
        backend.connect_child_exit();
        backend
    }

    pub(super) fn widget(&self) -> gtk::Widget {
        self.terminal.clone().upcast()
    }

    pub(super) fn set_event_handler(&self, handler: EventHandler) {
        *self.event_handler.borrow_mut() = Some(handler);
    }

    pub(super) fn spawn(&self, arguments: &[String], working_directory: &str) {
        if self.state.replace(ProcessState::Starting) != ProcessState::Idle {
            return;
        }

        let cancellable = gio::Cancellable::new();
        *self.cancellable.borrow_mut() = Some(cancellable.clone());

        let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
        let environment = inherited_environment();
        let environment: Vec<&str> = environment.iter().map(String::as_str).collect();
        let state = self.state.clone();
        let stored_cancellable = self.cancellable.clone();
        let event_handler = self.event_handler.clone();
        let close_callback = self.close_callback.clone();

        self.terminal.spawn_async(
            vte::PtyFlags::DEFAULT,
            Some(working_directory),
            &arguments,
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
                            run_close_callback(&close_callback);
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

    pub(super) fn request_close<F>(&self, on_ready: F) -> bool
    where
        F: FnOnce() + 'static,
    {
        match self.state.get() {
            ProcessState::Idle | ProcessState::Exited | ProcessState::Failed => true,
            ProcessState::Closing(_) => false,
            ProcessState::Starting => {
                *self.close_callback.borrow_mut() = Some(Box::new(on_ready));
                self.state.set(ProcessState::Closing(None));
                if let Some(cancellable) = self.cancellable.borrow().as_ref() {
                    cancellable.cancel();
                }
                self.schedule_close_escalation();
                false
            }
            ProcessState::Running(pid) => {
                *self.close_callback.borrow_mut() = Some(Box::new(on_ready));
                self.state.set(ProcessState::Closing(Some(pid)));
                let _ = send_signal(pid, libc::SIGHUP);
                self.schedule_close_escalation();
                false
            }
        }
    }

    fn connect_child_exit(&self) {
        let state = self.state.clone();
        let event_handler = self.event_handler.clone();
        let close_callback = self.close_callback.clone();

        self.terminal.connect_child_exited(move |terminal, status| {
            terminal.set_input_enabled(false);

            match state.get() {
                ProcessState::Closing(_) => {
                    state.set(ProcessState::Exited);
                    run_close_callback(&close_callback);
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
        let close_callback = self.close_callback.clone();
        glib::timeout_add_local_once(FORCED_CLOSE_DELAY, move || {
            if matches!(state.get(), ProcessState::Closing(_)) {
                state.set(ProcessState::Exited);
                run_close_callback(&close_callback);
            }
        });
    }
}

fn emit_event(handler: &RefCell<Option<EventHandler>>, event: TerminalEvent) {
    if let Some(handler) = handler.borrow().as_ref() {
        handler(event);
    }
}

fn run_close_callback(callback: &RefCell<Option<CloseCallback>>) {
    if let Some(callback) = callback.borrow_mut().take() {
        callback();
    }
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
    use super::inherited_environment;

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
}
