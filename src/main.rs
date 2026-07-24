mod app;
mod terminal;
mod ui;

fn main() -> gtk::glib::ExitCode {
    app::run()
}
