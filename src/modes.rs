use std::fs;
use std::process::{Command, Stdio};

pub enum Mode {
    BatterySaver,
    Balanced,
    Performance,
}

impl Mode {
    pub fn from_str(name: &str) -> Option<Mode> {
        match name.to_lowercase().as_str() {
            "saver" | "battery" => Some(Mode::BatterySaver),
            "balanced" => Some(Mode::Balanced),
            "performance" => Some(Mode::Performance),
            _ => None,
        }
    }
}

pub fn apply_mode(mode: &Mode) {
    match mode {
        Mode::BatterySaver => {
            println!("[Mode] Applying Battery Saver mode...");
            set_cpu_governor("powersave");
        }
        Mode::Balanced => {
            println!("[Mode] Applying Balanced mode...");
            set_cpu_governor("ondemand");
        }
        Mode::Performance => {
            println!("[Mode] Applying Performance mode...");
            set_cpu_governor("performance");
        }
    }
}

fn set_cpu_governor(governor: &str) {
    let path = "/sys/devices/system/cpu";
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let cpu_path = entry.path();
            if cpu_path.is_dir() {
                if let Some(name) = cpu_path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit()) {
                        let governor_path = format!(
                            "{}/cpufreq/scaling_governor",
                            cpu_path.to_string_lossy()
                        );

                        let mut child = Command::new("sudo")
                            .arg("tee")
                            .arg(&governor_path)
                            .stdin(Stdio::piped())
                            .spawn()
                            .expect("Failed to launch tee");

                        if let Some(stdin) = child.stdin.as_mut() {
                            use std::io::Write;
                            stdin
                                .write_all(governor.as_bytes())
                                .expect("Failed to write governor");
                        }

                        let _ = child.wait();
                    }
                }
            }
        }
    }
}
