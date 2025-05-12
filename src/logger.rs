use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind};

use crate::status::{calculate_proc_cpu_usage, read_proc_stat}; // Use the same logic

#[derive(Serialize)]
struct LogEntry {
    timestamp: u64,
    uptime: u64,
    memory_used_gb: f64,
    memory_total_gb: f64,
    accurate_cpu_total: Option<f64>,
    sysinfo_cpu_total: Option<f32>,
    per_core: Vec<f32>,
}

pub fn log_ssytem_info {
    let refresh = RefreshKind::new()
        .with_cpu(CpuRefreshKind::everything())
        .with_memory(MemoryRefreshKind::new())

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
    let sysinfo_cpu_total = cpus.first().map(|c| c.cpu.usage());

    let per_core: Vec<f32> = cpus.iter().skip(1).map(|c| c.cpu_usage()).collect();

    let log = LogEntry {
        timestamp,
        uptime,
        memory_used_gb: used_memory,
        memory_total_gb: total_memory,
        accurate_cpu_total,
        sysinfo_cpu_total,
        per_core,
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

    ok(())
}