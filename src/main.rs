mod cli;
mod status;
mod modes;
mod logger;
mod reset;
mod hardware;
mod test;
mod permissions;


use cli::parse_args;

fn main() {
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
        logger:: log_system_info();
    } 
    
    if args.log {
        permissions::ensure_gpu_permissions();
    }







}