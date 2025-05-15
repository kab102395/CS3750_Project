mod gui; // <-- Add this

fn main() {
    // Parse CLI args and handle CLI path
    let args = parse_args();

    // If no CLI args, launch GUI
    if !(args.show_status || args.reset || args.log || args.selected_mode.is_some()) {
        gui::launch_gui().unwrap();
        return;
    }

    ensure_gpu_permissions();

    if args.show_status {
        status::print_system_status();
    }

    if let Some(mode_str) = args.selected_mode {
        if let Some(mode) = modes::Mode::from_str(&mode_str) {
            modes::apply_mode(&mode);
        } else {
            eprintln!("[!] Unknown mode '{}'. Valid options: battery, balance, gaming", mode_str);
        }
    }

    if args.reset {
        reset_to_default();
    }

    if args.log {
        logger::log_system_info();
    }
}
