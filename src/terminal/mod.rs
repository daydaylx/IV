mod vte_backend;

use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use vte_backend::VteBackend;

#[derive(Clone)]
pub(crate) struct Terminal {
    backend: VteBackend,
}

impl Terminal {
    pub(crate) fn new() -> Self {
        Self {
            backend: VteBackend::new(),
        }
    }

    pub(crate) fn widget(&self) -> gtk::Widget {
        self.backend.widget()
    }

    pub(crate) fn set_event_handler(&self, handler: Rc<dyn Fn(TerminalEvent)>) {
        self.backend.set_event_handler(handler);
    }

    pub(crate) fn start(&self) -> Result<Vec<StartupWarning>, TerminalError> {
        let launch = LaunchConfig::from_environment()?;
        let warnings = launch.warnings.clone();
        self.backend
            .spawn(&launch.arguments, &launch.working_directory);
        Ok(warnings)
    }

    pub(crate) fn copy(&self) -> Result<(), TerminalError> {
        self.backend.copy()
    }

    pub(crate) fn paste(&self) -> Result<(), TerminalError> {
        self.backend.paste()
    }

    pub(crate) fn focus(&self) {
        self.backend.focus();
    }

    pub(crate) fn connect_title_changed<F>(&self, handler: F)
    where
        F: Fn(&str) + 'static,
    {
        self.backend.connect_title_changed(handler);
    }

    pub(crate) fn request_close<F>(&self, on_ready: F) -> bool
    where
        F: FnOnce() + 'static,
    {
        self.backend.request_close(on_ready)
    }
}

#[derive(Debug)]
pub(crate) enum TerminalEvent {
    Started,
    SpawnFailed(TerminalError),
    Exited(ProcessExit),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProcessExit {
    Code(i32),
    Signal(i32),
    Unknown,
}

impl ProcessExit {
    pub(crate) fn successful(self) -> bool {
        self == Self::Code(0)
    }

    pub(crate) fn user_message(self) -> String {
        match self {
            Self::Code(code) => format!("Die Shell wurde mit Status {code} beendet."),
            Self::Signal(signal) => {
                format!("Die Shell wurde durch Signal {signal} beendet.")
            }
            Self::Unknown => "Die Shell wurde unerwartet beendet.".to_owned(),
        }
    }

    fn from_wait_status(status: i32) -> Self {
        if libc::WIFEXITED(status) {
            Self::Code(libc::WEXITSTATUS(status))
        } else if libc::WIFSIGNALED(status) {
            Self::Signal(libc::WTERMSIG(status))
        } else {
            Self::Unknown
        }
    }
}

#[derive(Debug)]
pub(crate) enum TerminalError {
    NoUsableShell,
    NoUsableWorkingDirectory,
    Spawn(gtk::glib::Error),
    ClipboardUnavailable,
    NoSelection,
}

impl TerminalError {
    pub(crate) fn user_message(&self) -> &'static str {
        match self {
            Self::NoUsableShell => "Es wurde keine ausführbare lokale Shell gefunden.",
            Self::NoUsableWorkingDirectory => "Es wurde kein gültiges Startverzeichnis gefunden.",
            Self::Spawn(_) => "Die lokale Shell konnte nicht gestartet werden.",
            Self::ClipboardUnavailable => "Die Zwischenablage ist nicht verfügbar.",
            Self::NoSelection => "Es ist kein Text zum Kopieren ausgewählt.",
        }
    }
}

impl fmt::Display for TerminalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.user_message())
    }
}

impl std::error::Error for TerminalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StartupWarning {
    ShellFallback,
    WorkingDirectoryFallback,
}

impl StartupWarning {
    pub(crate) fn user_message(self) -> &'static str {
        match self {
            Self::ShellFallback => {
                "Die konfigurierte Standardshell war ungültig; /bin/sh wird verwendet."
            }
            Self::WorkingDirectoryFallback => {
                "Das aktuelle Verzeichnis war ungültig; ein Ersatzverzeichnis wird verwendet."
            }
        }
    }
}

struct LaunchConfig {
    arguments: Vec<String>,
    working_directory: String,
    warnings: Vec<StartupWarning>,
}

impl LaunchConfig {
    fn from_environment() -> Result<Self, TerminalError> {
        let (program, shell_fallback) = resolve_shell(env::var_os("SHELL"), Path::new("/bin/sh"))?;
        let (working_directory, directory_fallback) =
            resolve_working_directory(env::current_dir().ok(), env::var_os("HOME"))?;

        let mut warnings = Vec::new();
        if shell_fallback {
            warnings.push(StartupWarning::ShellFallback);
        }
        if directory_fallback {
            warnings.push(StartupWarning::WorkingDirectoryFallback);
        }

        Ok(Self {
            arguments: vec![program],
            working_directory,
            warnings,
        })
    }
}

fn resolve_shell(
    configured: Option<OsString>,
    fallback: &Path,
) -> Result<(String, bool), TerminalError> {
    if let Some(configured) = configured.as_deref()
        && let Some(shell) = usable_executable(Path::new(configured))
    {
        return Ok((shell, false));
    }

    usable_executable(fallback)
        .map(|shell| (shell, true))
        .ok_or(TerminalError::NoUsableShell)
}

fn usable_executable(path: &Path) -> Option<String> {
    if !path.is_absolute() {
        return None;
    }

    let metadata = fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return None;
    }

    path.to_str().map(str::to_owned)
}

fn resolve_working_directory(
    current: Option<PathBuf>,
    home: Option<OsString>,
) -> Result<(String, bool), TerminalError> {
    if let Some(current) = current.as_deref()
        && let Some(directory) = usable_directory(current)
    {
        return Ok((directory, false));
    }

    let fallback = home
        .as_deref()
        .and_then(|path| usable_directory(Path::new(path)))
        .or_else(|| usable_directory(Path::new("/")))
        .ok_or(TerminalError::NoUsableWorkingDirectory)?;

    Ok((fallback, true))
}

fn usable_directory(path: &Path) -> Option<String> {
    if !path.is_absolute() || !path.is_dir() {
        return None;
    }

    path.to_str().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_shell_is_used_when_executable() {
        let (shell, used_fallback) =
            resolve_shell(Some(OsString::from("/bin/sh")), Path::new("/bin/false"))
                .expect("/bin/sh should be available in the test environment");

        assert_eq!(shell, "/bin/sh");
        assert!(!used_fallback);
    }

    #[test]
    fn invalid_shell_uses_fallback() {
        let (shell, used_fallback) =
            resolve_shell(Some(OsString::from("relative-shell")), Path::new("/bin/sh"))
                .expect("/bin/sh should be available in the test environment");

        assert_eq!(shell, "/bin/sh");
        assert!(used_fallback);
    }

    #[test]
    fn missing_shell_and_fallback_is_an_error() {
        let result = resolve_shell(
            Some(OsString::from("/path/that/does/not/exist")),
            Path::new("/another/missing/path"),
        );

        assert!(matches!(result, Err(TerminalError::NoUsableShell)));
    }

    #[test]
    fn invalid_current_directory_uses_home() {
        let (directory, used_fallback) = resolve_working_directory(
            Some(PathBuf::from("/path/that/does/not/exist")),
            Some(OsString::from("/")),
        )
        .expect("/ should be available in the test environment");

        assert_eq!(directory, "/");
        assert!(used_fallback);
    }

    #[test]
    fn wait_status_decodes_exit_and_signal() {
        assert_eq!(ProcessExit::from_wait_status(7 << 8), ProcessExit::Code(7));
        assert_eq!(
            ProcessExit::from_wait_status(libc::SIGTERM),
            ProcessExit::Signal(15)
        );
    }
}
