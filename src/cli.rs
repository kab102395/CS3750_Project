use clap::{Arg, ArgAction, Command};

pub struct CliArgs {
    pub show_status: bool,
    pub selected_mode: Option<String>,
    pub reset: bool,
    pub log: bool,
}

pub fn parse_args() -> CliArgs {
    let matches = Command::new("Steam Deck Optimizer")
        .version("0.1.0")
        .author("Kyle Anthony Barrett")
        .about("Optimizes Steam Deck performance settings")
        .arg(
            Arg::new("status")
                .long("status")
                .help("Displays current system resource usage")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_parser(["gaming", "balance", "battery"])
                .help("Applies a performance mode"),
        )
        .arg(
            Arg::new("reset")
                .long("reset")
                .help("Restores system settings to default")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("log")
                .long("log")
                .help("Logs current system info to a file")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    CliArgs {
        show_status: matches.get_flag("status"),
        selected_mode: matches.get_one::<String>("mode").cloned(),
        reset: matches.get_flag("reset"),
        log: matches.get_flag("log"),
    }
}
