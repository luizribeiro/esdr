#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod radio;
mod ui;

fn main() -> ! {
    ui::run();
}
