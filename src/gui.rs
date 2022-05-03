// Copyright (c) 2022 aiocat
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::injector;
use crate::msgbox;

use eframe::{egui, egui::containers::ScrollArea, epaint};
use rfd::FileDialog;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use sysinfo::{PidExt, ProcessExt, System, SystemExt};

#[derive(Debug, std::cmp::PartialEq)]
enum InjectionMethods {
    CreateRemoteThread,
    RtlCreateUserThread,
}

// this struct holds application data for window lifecycle
pub struct DLLCrabWindow {
    pid: String,
    dll_name: String,
    dll_path: String,
    process_filter: String,
    system: System,
    processes: HashMap<u32, String>,
    close_after_injection: bool,
    selected_method: InjectionMethods,
}

// this function runs a new egui instance
pub fn draw_window() {
    // window options
    let options = eframe::NativeOptions {
        resizable: false,
        decorated: false,
        initial_window_size: Some(egui::Vec2 { x: 300.0, y: 450.0 }),
        ..Default::default()
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

impl Default for DLLCrabWindow {
    fn default() -> Self {
        let mut data = Self {
            pid: String::from("0"),
            dll_name: String::from("None"),
            dll_path: String::new(),
            process_filter: String::from("Filter"),
            system: System::new_all(),
            processes: HashMap::new(),
            close_after_injection: false,
            selected_method: InjectionMethods::CreateRemoteThread,
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
        // check if ends with dll
        if Path::new(&self.dll_path)
            .extension()
            .unwrap_or_else(|| OsStr::new(""))
            != "dll"
        {
            unsafe {
                msgbox::error("Library path is invalid. Please select a library to continue...");
            };
            return;
        }

        // check pid format
        let pid = self.pid.parse::<u32>();
        if pid.is_err() {
            unsafe {
                msgbox::error("PID format is invalid. Please check your input!");
            };
            return;
        }

        // run injector
        let pid: u32 = pid.unwrap();
        let function_to_use = match self.selected_method {
            InjectionMethods::CreateRemoteThread => injector::inject_create_remote_thread,
            InjectionMethods::RtlCreateUserThread => injector::inject_rtl_create_user_thread,
        };

        let result = function_to_use(pid, &self.dll_path);

        // check result
        unsafe {
            if !result {
                msgbox::error("Injection failed. Maybe PID is invalid?");
            } else {
                msgbox::info("Library is injected to the process.");
            }
        }
    }
}

// import eframe's lifecycle
impl eframe::App for DLLCrabWindow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let main_frame = egui::containers::Frame {
            rounding: egui::Rounding::none(),
            shadow: epaint::Shadow {
                extrusion: 0.0,
                color: egui::Color32::BLACK,
            },
            ..egui::containers::Frame::window(&egui::Style::default())
        };

        // top panel
        egui::TopBottomPanel::top("top")
            .frame(main_frame)
            .show(ctx, |ui: &mut egui::Ui| {
                ui.horizontal(|ui: &mut egui::Ui| {
                    if ui.button("X").clicked() {
                        frame.quit();
                    }

                    let item = egui::menu::bar(ui, |ui: &mut egui::Ui| {
                        ui.heading("DLL Crab");
                    });

                    if item.response.hovered() {
                        frame.drag_window();
                    }
                });
            });

        // bottom panel
        egui::TopBottomPanel::bottom("bottom")
            .frame(main_frame)
            .show(ctx, |ui: &mut egui::Ui| {
                ui.small("v1.0.0");
                egui::menu::bar(ui, |ui: &mut egui::Ui| {
                    ui.hyperlink_to("Source Code", "https://github.com/aiocat/dll-crab");
                    ui.hyperlink_to(
                        "Credits",
                        "https://github.com/aiocat/dll-crab/graphs/contributors",
                    );
                    ui.hyperlink_to(
                        "License",
                        "https://github.com/aiocat/dll-crab/blob/main/LICENSE",
                    );
                });
            });

        // main part
        egui::CentralPanel::default()
            .frame(main_frame)
            .show(ctx, |ui: &mut egui::Ui| {
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

                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label("Injection Method: ");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.selected_method))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.selected_method,
                                InjectionMethods::CreateRemoteThread,
                                "CreateRemoteThread",
                            );
                            ui.selectable_value(
                                &mut self.selected_method,
                                InjectionMethods::RtlCreateUserThread,
                                "RtlCreateUserThread",
                            );
                        });
                });

                // display buttons as inline-block
                ui.horizontal(|ui: &mut egui::Ui| {
                    // open dll file dialog
                    if ui.button("Open DLL").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("Dynamic Library", &["dll"])
                            .pick_file()
                        {
                            self.dll_name = path.file_name().unwrap().to_str().unwrap().to_owned();
                            self.dll_path = path.display().to_string();
                        }
                    }

                    // inject dll
                    if ui.button("Inject").clicked() {
                        self.inject();

                        if self.close_after_injection {
                            frame.quit();
                        }
                    }

                    // set close_after_injection
                    ui.checkbox(&mut self.close_after_injection, "Close After Injection");
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
                            if process.name().to_lowercase().contains(&self.process_filter) {
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
                        let row_height =
                            ui.fonts().row_height(&font_id) + ui.spacing().item_spacing.y;

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
