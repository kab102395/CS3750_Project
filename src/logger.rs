use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::path::Path;
use serde_json::Value;
use serde::Serialize;
use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind};

use crate::status::{calculate_proc_cpu_usage, read_proc_stat}; // Use the same logic
use crate::hardware::get_gpu_info;

#[derive(Serialize)]
struct LogEntry {
    timestamp: u64,
    uptime: u64,
    memory_used_gb: f64,
    memory_total_gb: f64,
    accurate_cpu_total: Option<f64>,
    sysinfo_cpu_total: Option<f32>,
    per_core: Vec<f32>,

    // gpu metrics
    gpu_util_percent: Option<u32>,
    gpu_temp_celsius: Option<f32>,
    gpu_core_clock_mhz: Option<u32>,
    gpu_mem_clock_mhz: Option<u32>,
}

pub fn log_system_info() {
    let refresh = RefreshKind::new()
        .with_cpu(CpuRefreshKind::everything())
        .with_memory(MemoryRefreshKind::new());

    let mut sys = System::new_with_specifics(refresh);
    sys.refresh_cpu();
    std::thread::sleep(std::time::Duration::from_millis(1000));
    sys.refresh_cpu();
    sys.refresh_memory();

    //time
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let uptime = System::uptime();

    //memory
    let total_memory = sys.total_memory() as f64 /1_048_576.0;
    let used_memory = sys.used_memory() as f64 / 1_048_576.0;

    //CPU
    let cpus = sys.cpus();
    let accurate_cpu_total = calculate_proc_cpu_usage();
    let sysinfo_cpu_total = cpus.first().map(|c| c.cpu_usage());
    let per_core: Vec<f32> = cpus.iter().skip(1).map(|c| c.cpu_usage()).collect();

    //GPU
    let (gpu_util_percent, gpu_temp_celsius, gpu_core_clock_mhz, gpu_mem_clock_mhz) = get_gpu_info();

    let log = LogEntry {
        timestamp,
        uptime,
        memory_used_gb: used_memory,
        memory_total_gb: total_memory,
        accurate_cpu_total,
        sysinfo_cpu_total,
        per_core,

        gpu_util_percent,
        gpu_temp_celsius,
        gpu_core_clock_mhz,
        gpu_mem_clock_mhz,
    };

    //write to file
    if let Err(e) = save_log(&log) {
        eprintln!("Error writing log: {}", e);
    } else {
        println!("System info loffed at timestamp: {}", timestamp);
    }
}

fn save_log(entry: &LogEntry) -> std::io::Result<()> {
    create_dir_all("logs")?;

    let filename = format!("logs/system_log_{}.json", entry.timestamp);
    let path = Path::new(&filename);
    let mut file = File::create(path)?;
    let data = serde_json::to_string_pretty(entry).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}

fn read_u32_from_file(path: &str) -> Option<u32> {
    std::fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<u32>()
        .ok()
}

fn read_hwmon_temp() -> Option<f32> {
    let pattern = "/sys/class/drm/card0/device/hwmon*/temp1_input";
    let paths = glob::glob(pattern).ok()?;

    for path in paths.flatten() {
        if let Ok(temp_raw) = std::fs::read_to_string(&path) {
            if let Ok(val) = temp_raw.trim().parse::<u32>() {
                return Some(val as f32 / 1000.0); 
            }
        }
    }
    None
}



/// Read and pretty-print the most recent system log.
pub fn read_most_recent_log_pretty() -> Option<String> {
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        return None;
    }

    let mut entries = fs::read_dir(log_dir).ok()?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
        .collect::<Vec<_>>();

    entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
    let latest = entries.pop()?;

    let raw = fs::read_to_string(latest.path()).ok()?;
    let parsed: Value = serde_json::from_str(&raw).ok()?;
    serde_json::to_string_pretty(&parsed).ok()
}
