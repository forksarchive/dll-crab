// Copyright (c) 2022 aiocat
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use rfd;

// this function runs a new egui instance
pub fn draw_window() {
    // window options
    let mut options = eframe::NativeOptions::default();
    options.resizable = false;
    options.initial_window_size = Some(egui::Vec2 { x: 300.0, y: 100.0 });

    // draw window
    eframe::run_native(
        "DLL Crab",
        options,
        Box::new(|ctx: &eframe::CreationContext| {
            let mut style = egui::Style::default();
            style.visuals.dark_mode = true;
            ctx.egui_ctx.set_style(style);

            Box::new(DLLCrabWindow::default())
        }),
    );
}

// this struct holds application data for window lifecycle
pub struct DLLCrabWindow {
    name: String,
    dll_name: String,
    dll_path: String,
}

impl Default for DLLCrabWindow {
    fn default() -> Self {
        Self {
            name: String::from("..."),
            dll_name: String::from("..."),
            dll_path: String::new(),
        }
    }
}

// import eframe's lifecycle
impl eframe::App for DLLCrabWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // title
            ui.heading("DLL Crab v1.0.0");

            // dll name label
            ui.horizontal(|ui| {
                ui.label("Selected DLL: ");
                ui.label(format!("{}", self.dll_name));
            });

            // application pid textbox
            ui.horizontal(|ui| {
                ui.label("Application: ");
                ui.text_edit_singleline(&mut self.name);
            });

            // display buttons as inline-block
            ui.horizontal(|ui| {
                // open dll file dialog
                if ui.button("Open DLL").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.dll_name = path.file_name().unwrap().to_str().unwrap().to_owned();
                        self.dll_path = path.display().to_string();

                        println!("{}", self.dll_path);
                    }
                }

                // inject dll
                if ui.button("Inject").clicked() {
                    println!("injected!")
                }
            });
        });
    }
}
