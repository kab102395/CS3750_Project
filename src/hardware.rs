use std::fs;
use std::io;
use std::process::Command;
use std::path::Path;
use glob::glob;

#[derive(Debug)]
struct AMDGPUStats {
    gpu_util_percent: Option<u32>,      // GPU core utilization (%)
    vram_util_percent: Option<u32>,     // VRAM controller utilization (%)
    core_clock_mhz: Option<u32>,        // Current core clock (MHz)
    memory_clock_mhz: Option<u32>,      // Current memory clock (MHz)
    temperature_c: Option<f32>,         // GPU temperature (Celsius)
    voltage_mv: Option<u32>,            // GPU core voltage (millivolts)
    fan_rpm: Option<u32>,               // Fan speed (RPM)
    power_watts: Option<f32>,           // Power draw (Watts)
    vram_used_bytes: Option<u64>,       // VRAM used (bytes)
    vram_total_bytes: Option<u64>,      // Total VRAM (bytes)
    gtt_used_bytes: Option<u64>,        // GTT (system memory) used (bytes)
    gtt_total_bytes: Option<u64>,       // Total GTT size (bytes)
    vis_vram_used_bytes: Option<u64>,   // Visible VRAM used (bytes)
    vis_vram_total_bytes: Option<u64>,  // Total visible VRAM (bytes)
}

/// Collects stats for the first AMD GPU found on the system.
fn collect_amdgpu_stats() -> io::Result<AMDGPUStats> {
    // 1. Find an AMD GPU card under /sys/class/drm
    let drm_path = "/sys/class/drm";
    let mut card_path: Option<String> = None;
    for entry in fs::read_dir(drm_path)? {
        let entry = entry?;
        let name = entry.file_name().into_string().unwrap_or_default();
        // We're looking for directories named "card0", "card1", etc (not connectors like card0-DP-1)
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        // Check vendor ID to ensure it's an AMD GPU (vendor 0x1002)
        let vendor_file = format!("{}/{}/device/vendor", drm_path, name);
        if let Ok(vendor_id) = fs::read_to_string(vendor_file) {
            if vendor_id.trim() == "0x1002" {
                card_path = Some(format!("{}/{}", drm_path, name));
                break;
            }
        }
    }
    let card_path = card_path.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No AMD GPU card found"))?;

    // 2. Attempt to read debugfs amdgpu_pm_info (with sudo fallback if needed)
    let mut dbg_gpu_util = None;
    let mut dbg_vram_util = None;
    let mut dbg_temp = None;
    let mut dbg_sclk = None;
    let mut dbg_mclk = None;
    // The debugfs entries are under /sys/kernel/debug/dri/*/amdgpu_pm_info
    if let Ok(mut glob_iter) = glob("/sys/kernel/debug/dri/*/amdgpu_pm_info") {
        for path in glob_iter.flatten() {
            // We could match the correct dri<N> by comparing with our card if needed.
            // Here, just try the first one that exists for AMDGPU.
            let dbg_path = path.to_string_lossy().to_string();
            // Try normal read first
            let content = fs::read_to_string(&dbg_path).or_else(|err| {
                if err.kind() == io::ErrorKind::PermissionDenied {
                    // Fallback: use `sudo cat` to read, capturing output
                    let output = Command::new("sudo")
                        .arg("cat")
                        .arg(&dbg_path)
                        .output()?;
                    if output.status.success() {
                        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
                    } else {
                        Err(io::Error::new(io::ErrorKind::Other, "sudo cat failed"))
                    }
                } else {
                    Err(err)
                }
            });
            if let Ok(text) = content {
                for line in text.lines() {
                    let line = line.trim();
                    if line.starts_with("GPU Load") {
                        // e.g., "GPU Load: 75 %"
                        if let Some(percent_str) = line.split_whitespace().nth(2) {
                            dbg_gpu_util = percent_str.parse::<u32>().ok();
                        }
                    } else if line.startsWith("MEM Load") {
                        if let Some(percent_str) = line.split_whitespace().nth(2) {
                            dbg_vram_util = percent_str.parse::<u32>().ok();
                        }
                    } else if line.starts_with("GPU Temperature") {
                        // e.g., "GPU Temperature: 65 C"
                        if let Some(temp_str) = line.split_whitespace().nth(2) {
                            if let Ok(temp_val) = temp_str.parse::<u32>() {
                                dbg_temp = Some(temp_val as f32);  // degrees C
                            }
                        }
                    } else if line.contains("(SCLK)") && !line.contains("PSTATE") && dbg_sclk.is_none() {
                        // e.g., "1200 MHz (SCLK)"
                        let parts: Vec<_> = line.split_whitespace().collect();
                        if parts.len() >= 3 && parts[1] == "MHz" {
                            if let Ok(freq) = parts[0].parse::<u32>() {
                                dbg_sclk = Some(freq);
                            }
                        }
                    } else if line.contains("(MCLK)") && !line.contains("PSTATE") && dbg_mclk.is_none() {
                        // e.g., "1000 MHz (MCLK)"
                        let parts: Vec<_> = line.split_whitespace().collect();
                        if parts.len() >= 3 && parts[1] == "MHz" {
                            if let Ok(freq) = parts[0].parse::<u32>() {
                                dbg_mclk = Some(freq);
                            }
                        }
                    }
                }
                break; // use the first amdgpu_pm_info we successfully read
            }
        }
    }

    // 3. Read sysfs entries for detailed metrics
    // Base device path (PCI device path) for the card
    let dev_path = format!("{}/device", card_path);
    // HWMon sensor path - assume one hwmon device under the GPU device
    let mut hwmon_path = None;
    if let Ok(entries) = fs::read_dir(format!("{}/hwmon", dev_path)) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.file_name().is_some() {
                hwmon_path = Some(p.display().to_string());
                break;
            }
        }
    }

    // Prepare struct with all fields
    let mut stats = AMDGPUStats {
        gpu_util_percent: None,
        vram_util_percent: None,
        core_clock_mhz: None,
        memory_clock_mhz: None,
        temperature_c: None,
        voltage_mv: None,
        fan_rpm: None,
        power_watts: None,
        vram_used_bytes: None,
        vram_total_bytes: None,
        gtt_used_bytes: None,
        gtt_total_bytes: None,
        vis_vram_used_bytes: None,
        vis_vram_total_bytes: None,
    };

    // GPU utilization (busy percent)
    if let Some(val) = dbg_gpu_util {
        stats.gpu_util_percent = Some(val);
    } else {
        if let Ok(val) = fs::read_to_string(format!("{}/gpu_busy_percent", dev_path)) {
            stats.gpu_util_percent = val.trim().parse::<u32>().ok();
        }
    }
    // Memory controller utilization
    if let Some(val) = dbg_vram_util {
        stats.vram_util_percent = Some(val);
    } else {
        if let Ok(val) = fs::read_to_string(format!("{}/mem_busy_percent", dev_path)) {
            stats.vram_util_percent = val.trim().parse::<u32>().ok();
        }
    }
    // Clocks: core (MHz) and memory (MHz)
    if let Some(freq) = dbg_sclk {
        stats.core_clock_mhz = Some(freq);
    }
    if let Some(freq) = dbg_mclk {
        stats.memory_clock_mhz = Some(freq);
    }
    if stats.core_clock_mhz.is_none() {
        // If hwmon is available, read freq1_input (Hz)
        if let Some(ref hpath) = hwmon_path {
            if let Ok(val) = fs::read_to_string(format!("{}/freq1_input", hpath)) {
                if let Ok(hz) = val.trim().parse::<u64>() {
                    stats.core_clock_mhz = Some((hz / 1_000_000) as u32);
                }
            }
        }
    }
    if stats.memory_clock_mhz.is_none() {
        if let Some(ref hpath) = hwmon_path {
            if let Ok(val) = fs::read_to_string(format!("{}/freq2_input", hpath)) {
                if let Ok(hz) = val.trim().parse::<u64>() {
                    stats.memory_clock_mhz = Some((hz / 1_000_000) as u32);
                }
            }
        }
    }
    // Temperature (Celsius)
    if let Some(temp) = dbg_temp {
        stats.temperature_c = Some(temp);
    } else if let Some(ref hpath) = hwmon_path {
        if let Ok(val) = fs::read_to_string(format!("{}/temp1_input", hpath)) {
            if let Ok(millideg) = val.trim().parse::<u32>() {
                stats.temperature_c = Some(millideg as f32 / 1000.0);
            }
        }
    }
    // Voltage (mV)
    if let Some(ref hpath) = hwmon_path {
        if let Ok(val) = fs::read_to_string(format!("{}/in0_input", hpath)) {
            stats.voltage_mv = val.trim().parse::<u32>().ok();
        }
    }
    // Fan speed (RPM)
    if let Some(ref hpath) = hwmon_path {
        if let Ok(val) = fs::read_to_string(format!("{}/fan1_input", hpath)) {
            stats.fan_rpm = val.trim().parse::<u32>().ok();
        }
    }
    // Power draw (Watts). Prefer averaged value if available.
    if let Some(ref hpath) = hwmon_path {
        let mut microwatts: Option<u64> = None;
        if let Ok(val) = fs::read_to_string(format!("{}/power1_average", hpath)) {
            microwatts = val.trim().parse::<u64>().ok();
        } else if let Ok(val) = fs::read_to_string(format!("{}/power1_input", hpath)) {
            microwatts = val.trim().parse::<u64>().ok();
        }
        if let Some(uw) = microwatts {
            stats.power_watts = Some(uw as f32 / 1_000_000.0);
        }
    }
    // VRAM and GTT memory usage (bytes)
    let mem_files = [
        ("vram_total_bytes", "mem_info_vram_total"),
        ("vram_used_bytes", "mem_info_vram_used"),
        ("vis_vram_total_bytes", "mem_info_vis_vram_total"),
        ("vis_vram_used_bytes", "mem_info_vis_vram_used"),
        ("gtt_total_bytes", "mem_info_gtt_total"),
        ("gtt_used_bytes", "mem_info_gtt_used"),
    ];
    for &(field, filename) in &mem_files {
        let path = format!("{}/{}", dev_path, filename);
        if let Ok(val) = fs::read_to_string(&path) {
            if let Ok(num) = val.trim().parse::<u64>() {
                match field {
                    "vram_total_bytes" => stats.vram_total_bytes = Some(num),
                    "vram_used_bytes" => stats.vram_used_bytes = Some(num),
                    "vis_vram_total_bytes" => stats.vis_vram_total_bytes = Some(num),
                    "vis_vram_used_bytes" => stats.vis_vram_used_bytes = Some(num),
                    "gtt_total_bytes" => stats.gtt_total_bytes = Some(num),
                    "gtt_used_bytes" => stats.gtt_used_bytes = Some(num),
                    _ => { /* ignore unknown */ }
                }
            }
        }
    }

    Ok(stats)
}
