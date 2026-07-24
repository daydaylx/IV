mod app;
mod pane;
mod settings;
mod tab;
mod terminal;
mod ui;

fn main() -> gtk::glib::ExitCode {
    app::run()
}
