mod cli;
mod status;
mod modes;
mod logger;
mod permissions;
mod hardware;
mod gui;

use cli::parse_args;
use permissions::ensure_gpu_permissions;
use modes::{apply_mode, Mode, reset_to_default};
use status::print_system_status;
use logger::log_system_info;
use gui::launch_gui;

fn main() {
    // Ensure proper GPU access and udev setup before anything else
    ensure_gpu_permissions();

    // Parse command-line arguments
    let args = parse_args();

    // If no flags are passed, launch the GUI instead
    let no_flags = !args.show_status && args.selected_mode.is_none() && !args.reset && !args.log;
    if no_flags {
        if let Err(e) = launch_gui() {
            eprintln!("[GUI] Failed to launch: {}", e);
        }
        return;
    }

    // CLI logic
    if args.show_status {
        print_system_status();
    }

    if let Some(mode_str) = args.selected_mode {
        match Mode::from_str(&mode_str) {
            Some(mode) => apply_mode(&mode),
            None => eprintln!("[CLI] Unknown mode: '{}'", mode_str),
        }
    }

    if args.reset {
        reset_to_default();
    }

    if args.log {
        log_system_info();
    }
}
