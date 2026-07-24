use adw::prelude::*;

const APPLICATION_ID: &str = "io.github.daydaylx.IV";

pub(crate) fn run() -> gtk::glib::ExitCode {
    let application = adw::Application::builder()
        .application_id(APPLICATION_ID)
        .build();

    application.connect_activate(|application| {
        if let Some(window) = application.active_window() {
            window.present();
            return;
        }

        crate::ui::build_main_window(application);
    });

    application.run()
}
