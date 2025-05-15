mod cli;
mod status;
mod modes;
mod logger;
mod permissions;
mod hardware;
mod gui; // ← Make sure this is here

use cli::parse_args;
use permissions::ensure_gpu_permissions;
use modes::{apply_mode, Mode, reset_to_default};
use status::print_system_status;
use logger::log_system_info;
use gui::launch_gui; // ← Add this

fn main() {
    ensure_gpu_permissions();
    let args = parse_args();

    // If no flags passed, launch GUI instead
    if !args.show_status && args.selected_mode.is_none() && !args.reset && !args.log {
        if let Err(e) = launch_gui() {
            eprintln!("[GUI] Failed to launch: {}", e);
        }
        return;
    }

    if args.show_status {
        print_system_status();
    }

    if let Some(mode_str) = args.selected_mode {
        if let Some(mode) = Mode::from_str(&mode_str) {
            apply_mode(&mode);
        } else {
            eprintln!("Unknown mode: '{}'", mode_str);
        }
    }

    if args.reset {
        reset_to_default();
    }

    if args.log {
        log_system_info();
    }
}
