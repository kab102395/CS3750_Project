mod cli;
mod status;
mod modes;
mod logger;
mod reset;
mod hardware;
mod permissions;

use cli::parse_args;
use permissions::ensure_gpu_permissions;

fn main() {
    // Ensure GPU access permissions (udev, groups, debugfs)
    ensure_gpu_permissions();

    // Parse CLI arguments
    let args = parse_args();

    // Show system status
    if args.show_status {
        status::print_system_status();
    }

    // Apply selected performance mode if provided
    if let Some(mode_str) = args.selected_mode {
        if let Some(mode) = modes::Mode::from_str(&mode_str) {
            modes::apply_mode(&mode);
        } else {
            eprintln!("[!] Unknown mode '{}'. Valid options: battery, balance, gaming", mode_str);
        }
    }

    // Reset system to defaults
    if args.reset {
        reset::reset_to_default();
    }

    // Log current system metrics to JSON
    if args.log {
        logger::log_system_info();
    }
}
