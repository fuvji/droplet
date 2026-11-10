#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod gp;
mod input;
mod keys;
mod macros;
mod smart;
mod ui;
mod win;

fn main() {
    app::run();
}
