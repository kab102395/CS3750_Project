use eframe::egui;
use crate::modes::{apply_mode, Mode, reset_to_default};
use crate::status::print_system_status;
use crate::logger::log_system_info;
use libc;
use std::os::fd::AsRawFd;

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
                let output = capture_stdout(|| {
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
fn capture_stdout<F: FnOnce() + Send + 'static>(func: F) -> String {
    use std::io::Read;
    use os_pipe::pipe;
    use std::thread;

    let (reader, writer) = pipe().unwrap();
    let stdout = std::io::stdout();
    let stdout_lock = stdout.lock();
    let saved = stdout_lock.as_raw_fd();
    let writer_fd = writer.as_raw_fd();

    unsafe {
        libc::dup2(writer_fd, saved);
    }

    let handle = thread::spawn(func);

    drop(writer); // Close writer to avoid blocking
    let mut output = String::new();
    reader.take(64 * 1024).read_to_string(&mut output).unwrap();
    handle.join().unwrap();

    output
}
