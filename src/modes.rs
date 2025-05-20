use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::Write;

#[derive(Debug, Clone)]
pub enum Mode {
    BatterySaver,
    Balanced,
    Performance,
    Custom(String),
}

impl Mode {
    pub fn from_str(name: &str) -> Option<Mode> {
        match name.to_lowercase().as_str() {
            "battery" | "saver" => Some(Mode::BatterySaver),
            "balanced" => Some(Mode::Balanced),
            "performance" => Some(Mode::Performance),
            other => Some(Mode::Custom(other.to_string())),
        }
    }
}

pub fn apply_mode(mode: &Mode) {
    let available_governors = get_available_governors();
    println!("[Mode] Available governors: {:?}", available_governors);

    let picked = match mode {
        Mode::BatterySaver => find_first_match(&available_governors, &["powersave", "schedutil"]),
        Mode::Balanced => find_first_match(&available_governors, &["ondemand", "schedutil"]),
        Mode::Performance => find_first_match(&available_governors, &["performance"]),
        Mode::Custom(name) => {
            if available_governors.contains(name) {
                Some(name.clone())
            } else {
                eprintln!("[Mode] Governor '{}' is not supported on this system.", name);
                None
            }
        }
    };

    if let Some(governor) = picked {
        println!("[Mode] Applying governor: {}", governor);
        set_cpu_governor(&governor);
    } else {
        eprintln!("[Mode] No compatible governor found for the selected mode.");
    }
}

fn find_first_match(available: &[String], preferred: &[&str]) -> Option<String> {
    for p in preferred {
        if available.contains(&p.to_string()) {
            return Some(p.to_string());
        }
    }
    None
}

pub fn reset_to_default() {
    println!("[Reset] Reverting to default...");

    let fallback_governors = ["ondemand", "schedutil", "powersave", "performance"];
    let available = get_available_governors();

    for gov in &fallback_governors {
        if available.contains(&gov.to_string()) {
            if try_set_cpu_governor(gov) {
                println!("[Reset] Set default governor: {}", gov);
                return;
            }
        }
    }

    eprintln!("[Reset] Could not apply fallback governor.");
}

pub fn get_available_governors() -> Vec<String> {
    let path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors";
    fs::read_to_string(path)
        .map(|c| c.split_whitespace().map(|s| s.to_string()).collect())
        .unwrap_or_else(|_| vec![])
}

pub fn set_cpu_governor(governor: &str) {
    if !try_set_cpu_governor(governor) {
        eprintln!("[Governor] Failed to set: {}", governor);
    }
}

fn try_set_cpu_governor(governor: &str) -> bool {
    let cpu_root = Path::new("/sys/devices/system/cpu");
    if let Ok(entries) = fs::read_dir(cpu_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with("cpu") || !path.join("cpufreq").exists() {
                continue;
            }

            let gov_path = path.join("cpufreq/scaling_governor");
            if gov_path.exists() {
                let child = Command::new("sudo")
                    .arg("tee")
                    .arg(gov_path.to_string_lossy().to_string())
                    .stdin(Stdio::piped())
                    .spawn();

                if let Ok(mut child) = child {
                    if let Some(stdin) = child.stdin.as_mut() {
                        let _ = stdin.write_all(governor.as_bytes());
                    }
                    let _ = child.wait();
                }
            }
        }
        true
    } else {
        false
    }
}
