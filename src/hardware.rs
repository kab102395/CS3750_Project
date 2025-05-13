// src/hardware.rs
use std::fs;
use std::path::Path;
use std::process::Command;

use glob::glob;

pub fn get_gpu_info() -> (Option<u32>, Option<u32>, Option<u32>, Option<u32>) {
    let gpu_type = detect_gpu_vendor();

    match gpu_type.as_deref() {
        Some("NVIDIA") => read_nvidia_gpu(),
        Some("AMD") => read_amd_gpu(),
        Some("Intel") => read_intel_gpu(),
        _ => (None, None, None, None),
    }
}

fn detect_gpu_vendor() -> Option<String> {
    let output = Command::new("lspci").arg("-nn").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);

    if text.contains("NVIDIA") {
        Some("NVIDIA".to_string())
    } else if text.contains("AMD") || text.contains("ATI") {
        Some("AMD".to_string())
    } else if text.contains("Intel") {
        Some("Intel".to_string())
    } else {
        None
    }
}

fn read_amd_gpu() -> (Option<u32>, Option<u32>, Option<u32>, Option<u32>) {
    let pattern = "/sys/kernel/debug/dri/*/amdgpu_pm_info";
    let paths = glob(pattern).ok()?;

    for entry in paths.flatten() {
        if let Ok(content) = fs::read_to_string(&entry) {
            let mut util = None;
            let mut temp = None;
            let mut sclk = None;
            let mut mclk = None;

            for line in content.lines() {
                if line.contains("GPU Load") {
                    util = line.split_whitespace().nth(2)?.trim().parse().ok();
                }
                if line.contains("GPU Temperature") {
                    temp = line.split_whitespace().nth(2)?.trim().parse().ok();
                }
                if line.contains("SCLK") && sclk.is_none() {
                    sclk = line.split_whitespace().nth(0)?.trim().parse().ok();
                }
                if line.contains("MCLK") && mclk.is_none() {
                    mclk = line.split_whitespace().nth(0)?.trim().parse().ok();
                }
            }
            return (util, temp, sclk, mclk);
        }
    }

    (None, None, None, None)
}

fn read_nvidia_gpu() -> (Option<u32>, Option<u32>, Option<u32>, Option<u32>) {
    let output = Command::new("nvidia-smi")
        .args(&["--query-gpu=utilization.gpu,temperature.gpu,clocks.sm,clocks.mem", "--format=csv,noheader,nounits"])
        .output()
        .ok()?;

    let data = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = data.trim().split(',').collect();
    if parts.len() != 4 {
        return (None, None, None, None);
    }

    let util = parts[0].trim().parse().ok();
    let temp = parts[1].trim().parse().ok();
    let core_clk = parts[2].trim().parse().ok();
    let mem_clk = parts[3].trim().parse().ok();

    (util, temp, core_clk, mem_clk)
}

fn read_intel_gpu() -> (Option<u32>, Option<u32>, Option<u32>, Option<u32>) {
    (None, None, None, None)
}
