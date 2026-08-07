#![windows_subsystem = "windows"]

mod audio;
mod gui;
mod logic;
mod settings;

use gui::AppFrame;

fn main() {
    wxdragon::main(|_app| {
        let mut frame = AppFrame::new();
        frame.setup_events();
        frame.show();
    })
    .unwrap();
}
