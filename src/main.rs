#![windows_subsystem = "windows"]

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "en-US");

mod audio;
mod db;
mod gui;
mod i18n;
mod logic;
mod screens;
mod settings;
mod updater;
mod visuals;

use gui::AppFrame;

fn main() {
    let settings = settings::Settings::load();
    rust_i18n::set_locale(settings.language.code());

    wxdragon::main(|_app| {
        let mut frame = AppFrame::new();
        frame.setup_events();
        frame.show();
    })
    .unwrap();
}
