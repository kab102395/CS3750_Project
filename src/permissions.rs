use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::{Command, Stdio};
use std::os::unix::fs::MetadataExt;
use std::env;
use nix::unistd::getgid;

pub fn ensure_gpu_permissions() {
    // 1. Check if user is in the "video" group
    if let Ok(groups_output) = Command::new("groups").output() {
        let groups = String::from_utf8_lossy(&groups_output.stdout);
        if !groups.contains("video") {
            println!("\n[!] This program needs GPU access (group: video)");
            println!("[+] Attempting to add you to the 'video' group...");

            let username = env::var("USER").unwrap_or_else(|_| "unknown".into());
            let _ = Command::new("sudo")
                .args(&["usermod", "-aG", "video", &username])
                .status()
                .expect("Failed to run usermod");

            println!("[+] Added to group. Please reboot or logout/login to apply.");
        }
    }

    // 2. Udev rules
    let udev_path = "/etc/udev/rules.d/99-gpu-permissions.rules";
    let desired_rules = r#"
# Allow all card* GPU devices to be accessed by users in the video group
KERNEL=="card*", GROUP="video", MODE="0660"
KERNEL=="renderD*", GROUP="video", MODE="0660"
SUBSYSTEM=="drm", GROUP="video", MODE="0660"
"#;

    let needs_write = match fs::read_to_string(udev_path) {
        Ok(content) => !content.contains("KERNEL==\"card*\""),
        Err(_) => true,
    };

    if needs_write {
        println!("[+] Creating or updating udev rule for GPU permissions...");

        let mut child = Command::new("sudo")
            .arg("tee")
            .arg(udev_path)
            .stdin(Stdio::piped())
            .spawn()
            .expect("sudo tee failed");

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(desired_rules.as_bytes())
                .expect("Failed to write to sudo tee");
        }

        child.wait().expect("Failed to wait on sudo tee");

        println!("[+] Reloading udev rules...");
        let _ = Command::new("sudo")
            .args(&["udevadm", "control", "--reload-rules"])
            .status();
        let _ = Command::new("sudo")
            .args(&["udevadm", "trigger"])
            .status();
    }

    // 3. Ensure debugfs is mounted (optional but helps access GPU info)
    let debugfs_check = Command::new("mountpoint")
        .arg("/sys/kernel/debug")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !debugfs_check {
        println!("[+] Mounting debugfs...");
        let _ = Command::new("sudo")
            .args(&["mount", "-t", "debugfs", "none", "/sys/kernel/debug"])
            .status();
    }

    // 4. Runtime Group and permission fix on amdgpu_pm_info
    let pm_info = "/sys/kernel/debug/dri/0000:c6:00.0/amdgpu_pm_info";
    if let Ok(metadata) = fs::metadata(pm_info) {
        let file_gid = metadata.gid();
        let user_gid = getgid().as_raw();

        if file_gid != user_gid {
            println!("[+] Fixing amdgpu_pm_info permissions for runtime access...");

            let _ = Command::new("sudo")
                .args(&["chgrp", "video", pm_info])
                .status();
            
            let _ = Command::new("sudo")
                .args(&["chmod", "660", pm_info])
                .status();
        }
    }
}
