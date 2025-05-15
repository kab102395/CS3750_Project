use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::Write;

#[derive(Debug)]
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
    let available_governors = get_available_governors();
    println!("[Mode] Available governors: {:?}", available_governors);

    match mode {
        Mode::BatterySaver => {
            for g in ["powersave", "schedutil"].iter() {
                if available_governors.contains(&g.to_string()) {
                    println!("[Mode] Applying Battery Saver mode with '{}'...", g);
                    set_cpu_governor(g);
                    return;
                }
            }
        }
        Mode::Balanced => {
            for g in ["ondemand", "schedutil"].iter() {
                if available_governors.contains(&g.to_string()) {
                    println!("[Mode] Applying Balanced mode with '{}'...", g);
                    set_cpu_governor(g);
                    return;
                }
            }
        }
        Mode::Performance => {
            if available_governors.contains(&"performance".to_string()) {
                println!("[Mode] Applying Performance mode with 'performance'...");
                set_cpu_governor("performance");
                return;
            }
        }
    }

    eprintln!("[Mode] No suitable governor found for the selected mode.");
}

pub fn reset_to_default() {
    println!("[Reset] Reverting system settings to default mode...");

    let available_governors = get_available_governors();
    println!("[Reset] Available governors on this system: {:?}", available_governors);

    let fallback_governors = ["ondemand", "schedutil", "powersave", "performance"];

    for governor in fallback_governors.iter() {
        if available_governors.contains(&governor.to_string()) {
            if try_set_cpu_governor(governor) {
                println!("[Reset] CPU governor set to '{}'.", governor);
                return;
            }
        }
    }
    eprintln!("[Reset] Failed to set any compatible CPU governor.");
}

fn try_set_cpu_governor(governor: &str) -> bool {
    let cpu_dir = Path::new("/sys/devices/system/cpu");

    if let Ok(entries) = fs::read_dir(cpu_dir) {
        for entry in entries.flatten() {
            let cpu_path = entry.path();
            let name = cpu_path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with("cpu") || !cpu_path.is_dir() {
                continue;
            }
            let gov_path = cpu_path.join("cpufreq/scaling_governor");
            if gov_path.exists() {
                let child = Command::new("sudo")
                    .arg("tee")
                    .arg(gov_path.to_str().unwrap())
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
        return true;
    }

    false
}

pub fn set_cpu_governor(governor: &str) {
    if !try_set_cpu_governor(governor) {
        eprintln!("[Governor] Failed to set governor '{}'.", governor);
    }
}

pub fn get_available_governors() -> Vec<String> {
    let path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors";
    if Path::new(path).exists() {
        if let Ok(content) = fs::read_to_string(path) {
            return content
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
        }
    }
    vec![]
}
