use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn reset_to_default() {
    println!("[Reset] Reverting system settings to default mode...");

    let default_governors = ["ondemand", "schedutil"];

    for governor in &default_governors {
        if try_set_cpu_governor(governor) {
            println!("[Reset] CPU governor set to '{}'.", governor);
            return;
        }
    }

    eprintln!("[Reset] Failed to set any known default CPU governor.");
}

fn try_set_cpu_governor(governor: &str) -> bool {
    let cpu_dir = Path::new("/sys/devices/system/cpu");
    let mut success = false;

    if let Ok(entries) = fs::read_dir(cpu_dir) {
        for entry in entries.flatten() {
            let cpu_path = entry.path();
            let name = cpu_path.file_name().unwrap_or_default().to_string_lossy();

            if !name.starts_with("cpu") || !cpu_path.is_dir() {
                continue;
            }

            let gov_path = cpu_path.join("cpufreq/scaling_governor");
            if gov_path.exists() {
                if let Ok(mut child) = Command::new("sudo")
                    .arg("tee")
                    .arg(gov_path.to_str().unwrap())
                    .stdin(Stdio::piped())
                    .spawn()
                {
                    if let Some(stdin) = child.stdin.as_mut() {
                        let _ = stdin.write_all(governor.as_bytes());
                    }
                    let _ = child.wait();
                    success = true;
                }
            }
        }
    }

    success
}
