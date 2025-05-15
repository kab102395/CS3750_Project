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
use crate::hardware::get_gpu_info;

fn main() {
    ensure_gpu_permissions();

    let args = parse_args();

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
