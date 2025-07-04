use eframe::egui;
use crate::modes::{apply_mode, Mode, reset_to_default};
use crate::logger::{log_system_info, read_latest_log};
use crate::games::{discover_all_games, GameInfo};
use std::sync::mpsc;
use std::thread;

pub fn launch_gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Steam Deck Optimizer",
        options,
        Box::new(|_cc| Box::new(DeckOptimizerGui::default())),
    )
}

struct DeckOptimizerGui {
    status_output: String,
    selected_mode: Option<Mode>,
    status_requested: bool,
    status_result: Option<String>,
    status_receiver: Option<mpsc::Receiver<String>>,
    discovered_games: Vec<GameInfo>,
}

impl Default for DeckOptimizerGui {
    fn default() -> Self {
        Self {
            status_output: String::new(),
            selected_mode: None,
            status_requested: false,
            status_result: None,
            status_receiver: None,
            discovered_games: Vec::new(),
        }
    }
}

impl eframe::App for DeckOptimizerGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                
                ui.heading("Steam Deck Optimizer");

                // --- Mode Selector ---
                ui.separator();
                ui.label("Select Mode:");
                ui.horizontal(|ui| {
                    if ui.button("Battery Saver").clicked() {
                        apply_mode(&Mode::BatterySaver);
                        self.selected_mode = Some(Mode::BatterySaver);
                    }
                    if ui.button("Balanced").clicked() {
                        apply_mode(&Mode::Balanced);
                        self.selected_mode = Some(Mode::Balanced);
                    }
                    if ui.button("Performance").clicked() {
                        apply_mode(&Mode::Performance);
                        self.selected_mode = Some(Mode::Performance);
                    }
                });

                // --- Extra Governors ---
                ui.separator();
                ui.label("Or Select Specific CPU Governor:");
                for governor in crate::modes::get_available_governors() {
                    let label = format!("🛠 {}", governor);
                    if ui.button(label).clicked() {
                        apply_mode(&Mode::Custom(governor.clone()));
                        self.selected_mode = Some(Mode::Custom(governor));
                    }
                }

                if let Some(mode) = &self.selected_mode {
                    ui.label(format!("Last mode applied: {:?}", mode));
                }

                // --- System Status Section ---
                ui.separator();
                if ui.button("Show System Status").clicked() && !self.status_requested {
                    self.status_requested = true;
                    let (sender, receiver) = mpsc::channel();
                    thread::spawn(move || {
                        let output = read_latest_log().unwrap_or("[Error] No logs found.".to_string());
                        let _ = sender.send(output);
                    });
                    self.status_receiver = Some(receiver);
                }

                if let Some(ref rx) = self.status_receiver {
                    match rx.try_recv() {
                        Ok(output) => {
                            self.status_output = output;
                            self.status_receiver = None;
                            self.status_requested = false;
                        }
                        Err(mpsc::TryRecvError::Empty) => {
                            ui.label("Fetching system status...");
                            ctx.request_repaint();
                        }
                        Err(e) => {
                            self.status_output = format!("[Error] Channel error: {e}");
                            self.status_receiver = None;
                            self.status_requested = false;
                        }
                    }
                }

                if ui.button("Reset to Default").clicked() {
                    reset_to_default();
                }

                if ui.button("Log Current Stats").clicked() {
                    log_system_info();
                }

                ui.separator();
                ui.label("System Status Output:");
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.label(&self.status_output);
                    });

                // --- Game Detection ---
                ui.separator();
                if ui.button("Detect Installed Games").clicked() {
                    self.discovered_games = discover_all_games();
                }

                ui.separator();
                ui.heading("Detected Games");

                egui::ScrollArea::vertical()
                    .max_height(350.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for game in &self.discovered_games {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    if let Some(img_path) = &game.cover_image {
                                        if let Ok(img_data) = std::fs::read(img_path) {
                                            if let Ok(image) = egui_extras::image::load_image_bytes(&img_data) {
                                                let tex = ctx.load_texture(
                                                    &game.name,
                                                    image,
                                                    Default::default(),
                                                );
                                                ui.add(
                                                    egui::Image::new(&tex)
                                                        .fit_to_exact_size(egui::Vec2::new(64.0, 64.0)),
                                                );
                                            }
                                        }
                                    } else {
                                        ui.label("[No Cover]");
                                    }

                                    ui.vertical(|ui| {
                                        ui.label(format!("🎮 {}", game.name));
                                        ui.label(format!("Source: {}", game.source));
                                        if ui.button("Set Optimizations").clicked() {
                                            println!("Optimizations applied for: {}", game.name);
                                        }
                                    });
                                });
                            });
                            ui.add_space(6.0);
                        }
                    });
            });
    });
}
}