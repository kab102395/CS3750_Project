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
            set_cpu_govenor("ondemand");
        }
        Mode::Performance => {
            println!("[Mode] Applying Performance mode...");
            set_cpu_govenor("performance");
        }
    }
}

fn set_cpu_govenor(governor: &str) {
    let path = "/sys/devices/system/cpu";
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let cpu_path = entry.path();
            if cpu_path.is_dir() && cpu_path.file_name().unwrap_or_default().to_str().unwrap_or("").starts_with("cpu") {
                let _ = Command::new("sudo")
                    .arg("tee")
                    .arg(govenor_path)
                    .stdin(std::process:Stdio::piped())
                    .spawn()
                    .and_then(|mut child| {
                        if let Some(stdin) = child.stdin.as_mut() {
                            use std::io::Write;
                            stdin.write_all(govenor.as_bytes())?;
                        }
                        child.wait()?;
                        ok(())
                    }); 
            } 
            
        }
    }
}