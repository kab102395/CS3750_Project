use glob::glob;
use std::process::Command;
use std::fs;
use std::io;
use std::path::Path;

pub fn get_gpu_info() -> (Option<u32>, Option<f32>, Option<u32>, Option<u32>) {
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

fn read_amd_gpu() -> (Option<u32>, Option<f32>, Option<u32>, Option<u32>) {
    let paths = glob("/sys/kernel/debug/dri/*/amdgpu_pm_info").ok()?;
    for entry in paths.flatten() {
        if let Ok(content) = fs::read_to_string(&entry) {
            let mut util = None;
            let mut temp = None;
            let mut sclk = None;
            let mut mclk = None;

            for line in content.lines() {
                if line.contains("GPU Load") {
                    util = line.split_whitespace().nth(2).and_then(|v| v.parse().ok());
                }
                if line.contains("GPU Temperature") {
                    temp = line.split_whitespace().nth(2).and_then(|v| v.parse::<u32>().ok()).map(|v| v as f32);
                }
                if line.contains("SCLK") && sclk.is_none() {
                    sclk = line.split_whitespace().nth(0).and_then(|v| v.parse().ok());
                }
                if line.contains("MCLK") && mclk.is_none() {
                    mclk = line.split_whitespace().nth(0).and_then(|v| v.parse().ok());
                }
            }

            return (util, temp, sclk, mclk);
        }
    }

    (None, None, None, None)
}




fn read_nvidia_gpu() -> (Option<u32>, Option<f32>, Option<u32>, Option<u32>) {
    let output = Command::new("nvidia-smi")
        .args(&["--query-gpu=utilization.gpu,tempature.gpu,clocks.sm,clocks.mem","--format=csv.noheader,nounits"])
        .output()
        .ok();
    let output = match output {
            Some(data) => data,
            None => return (None, None, None, None),
    };
    let data = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = data.trim().split(',').collect();
    if parts.len() != 4 {
        return (None, None, None, None);
    }

    let util = parts[0].trim().parse().ok();
    let temp = parts[1].trim().parse::<f32>().ok();
    let core_clk = parts[2].trim().parse().ok();
    let mem_clk = parts[3].trim().parse().ok();

    (util, temp, core_clk, mem_clk)
}

fn read_intel_gpu() -> (Option<u32>, Option<f32>, Option<u32>, Option<u32>) {
    (None, None, None, None)
    // intel is limited in GPU file based access
}

pub fn read_u32_from_file(path: &str) -> Option<u32> {
    std::fs::read_to_string(path).ok()?.trim().parse::<u32>().ok()
}

pub fn read_hwmon_temp() -> Option<f32> {
    let pattern = "/sys/class/drm/card0/device/hwmon*/temp1_input";
    let paths = glob(pattern).ok()?;

    for path in paths.flatten() {
        if let Ok(temp_raw) = std::fs::read_to_string(&path) {
            if let Ok(val) = temp_raw.trim().parse::<u32>() {
                return Some(val as f32 / 1000.0);
            }
        }
    }

    None
}

fn read_from_debugfs(file: &str, key: &str) -> Option<u32> {
    let pattern = "/sys/kernel/debug/dri/*/amdgpu_pm_info";
    let paths = glob(pattern).ok()?;

    for path in paths.flatten() {
        if path.to_string_lossy().contains(file) {
            if let Ok(content) = fs::read_to_string(&path) {
                for line in content.lines() {
                    if line.contains(key) {
                        for token in line.split_whitespace() {
                            if let Ok(val) = token.parse::<u32>() {
                                return Some(val);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

