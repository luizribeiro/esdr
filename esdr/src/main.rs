#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod blocks;
mod consts;
mod param;
mod radio;
mod ui;

#[macro_use]
extern crate enum_dispatch;

#[macro_use]
extern crate derive_builder;

fn main() -> ! {
    ui::run();
}
