#![windows_subsystem = "windows"]

mod audio;
mod db;
mod gui;
mod logic;
mod screens;
mod settings;
mod updater;

use gui::AppFrame;

fn main() {
    wxdragon::main(|_app| {
        let mut frame = AppFrame::new();
        frame.setup_events();
        frame.show();
    })
    .unwrap();
}
