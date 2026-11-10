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
    // Optional learned timing prior from the offline combat lab. Missing or
    // invalid files leave the deterministic local brain unchanged.
    smart::load_profile();
    app::run();
}
