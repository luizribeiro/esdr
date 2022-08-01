#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod blocks;
mod radio;
mod ui;

#[macro_use]
extern crate enum_dispatch;

fn main() -> ! {
    ui::run();
}
