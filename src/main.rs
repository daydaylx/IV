mod app;
mod pane;
mod settings;
mod tab;
mod terminal;
mod ui;
mod workspace;

fn main() -> gtk::glib::ExitCode {
    app::run()
}
