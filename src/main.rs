// Copyright (c) 2022 aiocat
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gui;
pub mod injector;
pub mod msgbox;
pub mod spoof;

fn main() {
    gui::draw_window();
}
