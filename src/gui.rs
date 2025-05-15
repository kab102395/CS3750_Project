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
            if ui.button("Show System Status").clicked() {
                // Redirect stdout to a buffer for GUI
                let output = capture_stdout_threaded(|| {
                    print_system_status();
                });
                
                self.status_output = output;
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
fn capture_stdout_threaded<F: FnOnce() + Send + 'static>(f: F) -> String {
    let (reader, writer) = pipe().unwrap();
    let writer_fd = writer.as_raw_fd();
    let saved_fd = std::io::stdout().as_raw_fd();

    // Duplicate original stdout to restore later
    let saved = unsafe { libc::dup(saved_fd) };

    // Redirect stdout to writer
    use std::io::Write; // Add to your imports if not already

    let _ = std::io::stdout().flush(); // ⬅️ ADD THIS

    unsafe {
        libc::dup2(writer_fd, saved_fd);
    }


    let handle = thread::spawn(f);

    // Close the writer in this thread so the reader sees EOF
    drop(writer);

    let mut output = String::new();
    let mut reader_file = unsafe { File::from_raw_fd(reader) };
    let _ = reader_file.read_to_string(&mut output);



    handle.join().ok();

    // Restore original stdout
    unsafe {
        libc::dup2(saved, saved_fd);
        libc::close(saved);
    }

    output
}
