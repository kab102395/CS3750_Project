use std::process::Command;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::env;

pub fn ensure_gpu_permissions() {
    // Check if we're in the video group
    if let Ok(groups_output) = Command::new("groups").output() {
        let groups = String::from_utf8_lossy(&groups_output.stdout);
        if !groups.contains("video") {
            println!("\n[!] This program needs GPU access (group: video)");
            println!("[+] Attempting to add you to the 'video' group...");

            let username = env::var("USER").unwrap_or("unknown".into());
            let _ = Command::new("sudo")
                .args(&["usermod", "-aG", "video", &username])
                .status()
                .expect("Failed to run usermod");

            println!("[+] Added to group. Please reboot or logout/login to apply.");
        }
    }

    // Check if udev rule already exists
    let udev_path = "/etc/udev/rules.d/99-gpu-permissions.rules";
    let needs_write = match fs::read_to_string(udev_path) {
        Ok(content) => !content.contains("amdgpu"),
        Err(_) => true,
    };

    if needs_write {
        println!("[+] Creating udev rule to fix GPU permissions...");

        let rule = r#"KERNEL=="card*", GROUP="video", MODE="0660""#;
        let mut file = Command::new("sudo")
            .arg("tee")
            .arg(udev_path)
            .stdin(OpenOptions::new().write(true).open("/dev/null").unwrap())
            .spawn()
            .expect("sudo tee failed");

        file.stdin
            .as_mut()
            .unwrap()
            .write_all(rule.as_bytes())
            .expect("write failed");

        println!("[+] Reloading udev...");
        let _ = Command::new("sudo")
            .args(&["udevadm", "control", "--reload-rules"])
            .status();
        let _ = Command::new("sudo")
            .args(&["udevadm", "trigger"])
            .status();
    }
}
