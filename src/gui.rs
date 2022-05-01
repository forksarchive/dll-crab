// Copyright (c) 2022 aiocat
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::injector;
use eframe::{egui, egui::containers::ScrollArea};
use rfd::FileDialog;
use std::collections::HashMap;
use sysinfo::{PidExt, ProcessExt, System, SystemExt};

// this function runs a new egui instance
pub fn draw_window() {
    // window options
    let options = eframe::NativeOptions {
        resizable: false,
        initial_window_size: Some(egui::Vec2 { x: 300.0, y: 300.0 }),
        ..eframe::NativeOptions::default()
    };

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
    pid: String,
    dll_name: String,
    dll_path: String,
    process_filter: String,
    system: System,
    processes: HashMap<u32, String>,
}

impl Default for DLLCrabWindow {
    fn default() -> Self {
        let mut data = Self {
            pid: String::from("0"),
            dll_name: String::from("..."),
            dll_path: String::new(),
            process_filter: String::from("Filter"),
            system: System::new_all(),
            processes: HashMap::new(),
        };

        data.system.refresh_all();
        for (pid, process) in data.system.processes() {
            data.processes
                .insert(pid.as_u32(), process.name().to_string());
        }

        data
    }
}

impl DLLCrabWindow {
    pub fn inject(&self) {
        let pid = self.pid.parse::<u32>();

        if pid.is_err() {
            println!("Error!");
            return;
        }

        let pid: u32 = pid.unwrap();
        injector::inject_dll(pid, &self.dll_path);
    }
}

// import eframe's lifecycle
impl eframe::App for DLLCrabWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            // title
            ui.heading("Injection");
            ui.add_space(4.0);

            // dll name label
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label("Selected DLL: ");
                ui.label(&self.dll_name);
            });

            // application pid textbox
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label("Application PID: ");
                ui.text_edit_singleline(&mut self.pid);
            });

            // display buttons as inline-block
            ui.horizontal(|ui: &mut egui::Ui| {
                // open dll file dialog
                if ui.button("Open DLL").clicked() {
                    if let Some(path) = FileDialog::new().pick_file() {
                        self.dll_name = path.file_name().unwrap().to_str().unwrap().to_owned();
                        self.dll_path = path.display().to_string();
                    }
                }

                // inject dll
                if ui.button("Inject").clicked() {
                    self.inject()
                }
            });

            ui.add_space(8.0);
            ui.heading("Processes");
            ui.add_space(4.0);
            ui.horizontal(|ui: &mut egui::Ui| {
                // refresh list button
                if ui.button("Refresh").clicked() {
                    self.system.refresh_all();
                    self.process_filter = String::from("Filter");
                    self.processes = HashMap::new();
                    for (pid, process) in self.system.processes() {
                        self.processes
                            .insert(pid.as_u32(), process.name().to_string());
                    }
                }

                // filter list
                if ui.button("Filter").clicked() {
                    self.system.refresh_all();

                    self.processes = HashMap::new();
                    for (pid, process) in self.system.processes() {
                        if process.name().contains(&self.process_filter) {
                            self.processes
                                .insert(pid.as_u32(), process.name().to_string());
                        }
                    }
                }

                // filter list by process name textbox
                ui.text_edit_singleline(&mut self.process_filter);
            });

            // process list
            ui.add_space(4.0);
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show_viewport(ui, |ui: &mut eframe::egui::Ui, _| {
                    let font_id = egui::TextStyle::Body.resolve(ui.style());
                    let row_height = ui.fonts().row_height(&font_id) + ui.spacing().item_spacing.y;

                    ui.set_height(self.processes.len() as f32 * (row_height * 1.5));

                    for (pid, process) in &self.processes {
                        ui.horizontal(|ui| {
                            ui.label(pid.to_string());

                            // load pid
                            if ui.link(process).clicked() {
                                self.pid = pid.to_string();
                            }
                        });
                    }
                });
        });
    }
}
