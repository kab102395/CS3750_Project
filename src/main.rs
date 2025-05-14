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
    // Automatically ensure GPU permission rules are in place
    ensure_gpu_permissions();

    let args = parse_args();

    if args.show_status {
        status::print_system_status();
    }

    if let Some(mode) = args.selected_mode {
        modes::apply_mode(&mode);
    }

    if args.reset {
        reset::reset_to_default();
    }

    if args.log {
        logger::log_system_info();
    }
}
