use glob::glob;
use std::process::Command;
use std::fs;
use std::io;
use std::path::Path;

pub fn get_gpu_info() -> (Option<u32>, Option<f32>, Option<u32>, Option<u32>) {
    let gpu_type = dtect_gpu_vendor();

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
    let util = read_u32_from_file("/sys/class/drm/card0/device/gpu_busy_percent")
    .or_else(|| read_from_debugfs("amdgpu_pm_info", "GPU load"));
let temp = read_hwmon_temp();
let core_clk = read_u32_from_file("/sys/class/drm/card0/device/pp_cur_sclk")
    .or_else(|| read_from_debugfs("amdgpu_pm_info", "SCLK"));
let mem_clk = read_u32_from_file("/sys/class/drm/card0/device/pp_cur_mclk")
    .or_else(|| read_from_debugfs("amdgpu_pm_info", "MCLK"));

(util, temp, core_clk, mem_clk)
}

fn read_nvidia_gpu() -> (Option<u32>, Option<f32>, Option<u32>, Option<u32>) {
    let output = Command::new("nvidia-smi")
        .args(&["--query-gpu=utilization.gpu,tempature.gpu,clocks.sm,clocks.mem","--format=csv.noheader,nounits"])
        .output()
        .ok()?;

    let data = String::from_utf8_lossy(&output.stdout);


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


