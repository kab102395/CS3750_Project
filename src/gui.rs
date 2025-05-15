use eframe::egui;
use crate::modes::{apply_mode, Mode, reset_to_default};
use crate::status::print_system_status;
use crate::logger::log_system_info;
use libc;
use std::os::fd::AsRawFd;
use std::thread;
use nix::unistd::pipe;
use std::fs::File;
use std::io::Read;
use std::os::fd::FromRawFd;

pub fn launch_gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Steam Deck Optimizer",
        options,
        Box::new(|_cc| Box::new(DeckOptimizerGui::default())),
    )
}

#[derive(Default)]
struct DeckOptimizerGui {
    status_output: String,
    selected_mode: Option<Mode>,
    status_requested: bool,
    status_result: Option<String>,
    status_receiver: Option<std::sync::mpsc::Receiver<String>>,
}



impl eframe::App for DeckOptimizerGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Steam Deck Optimizer");

            // --- Mode Selector ---
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Select Mode:");

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

            if let Some(mode) = &self.selected_mode {
                ui.label(format!("Last mode applied: {:?}", mode));
            }

            // --- Actions ---
            ui.separator();
            if ui.button("Show System Status").clicked() && !self.status_requested {
                self.status_requested = true;
            
                let (sender, receiver) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let output = capture_stdout_threaded(print_system_status);
                    let _ = sender.send(output);
                });
            
                self.status_receiver = Some(receiver);
            }
            
            // ✅ Now properly handle received data
            if let Some(ref rx) = self.status_receiver {
                match rx.try_recv() {
                    Ok(output) => {
                        self.status_output = output;
                        self.status_receiver = None;
                        self.status_requested = false;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        // Still waiting
                        ui.label("Fetching system status...");
                        ctx.request_repaint(); // ❗ Only request when waiting
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.status_output = "[Error] Failed to receive system status.".into();
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

            // --- Status Output ---
            ui.separator();
            ui.label("System Status Output:");
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(&self.status_output);
            });
        });
    }
}

// Utility function to capture stdout temporarily for displaying logs in GUI
fn capture_stdout_threaded<F: FnOnce()>(f: F) -> String {
    let (reader, writer) = pipe().unwrap();
    let writer_fd = writer.as_raw_fd();
    let saved_fd = std::io::stdout().as_raw_fd();

    use std::io::Write;
    let _ = std::io::stdout().flush();

    let saved = unsafe { libc::dup(saved_fd) };
    println!("[DEBUG] Inside print_system_status");

    unsafe {
        libc::dup2(writer_fd, saved_fd);
    }

    // ✅ Run `f()` directly here in this thread!
    f();

    unsafe {
        libc::dup2(saved, saved_fd);
        libc::close(saved);
    }

    drop(writer); // Ensure writer is dropped so reader gets EOF

    let mut output = String::new();
    let mut reader_file = unsafe { File::from_raw_fd(reader) };
    let _ = reader_file.read_to_string(&mut output);

    output
}

