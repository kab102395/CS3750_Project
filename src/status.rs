use std::{fs::File, io::{BufRead, BufReader}, thread, time::Duration};
use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind};
use crate::hardware::get_gpu_info;

pub fn print_system_status() {
    // Initialize sysinfo
    let refresh = RefreshKind::new()
        .with_cpu(CpuRefreshKind::everything())
        .with_memory(MemoryRefreshKind::new());

    let mut sys = System::new_with_specifics(refresh);
    sys.refresh_cpu();
    thread::sleep(Duration::from_millis(1000));
    sys.refresh_cpu();
    sys.refresh_memory();

    println!("\n=== System Status ===");

    // Accurate Total CPU from /proc/stat
    if let Some(cpu_usage) = calculate_proc_cpu_usage() {
        println!("Accurate Total CPU (from /proc/stat): {:.1}%", cpu_usage);
    } else {
        println!("Accurate Total CPU: [error]");
    }

    // Uptime
    println!("Uptime: {} seconds", System::uptime());

    // Memory
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    println!(
        "Memory: {:.2} / {:.2} GB",
        used_memory as f64 / 1_048_576.0,
        total_memory as f64 / 1_048_576.0
    );

    // CPU: sysinfo total + per-core
    let cpus = sys.cpus();
    if let Some(total) = cpus.first() {
        println!("Sysinfo Reported Total CPU: {:>5.1}%", total.cpu_usage());
    }

    for (i, cpu) in cpus.iter().skip(1).enumerate() {
        println!("Core {:2}: {:>5.1}%", i, cpu.cpu_usage());
    }

    // GPU info 
    let (gpu_util, gpu_temp, gpu_core_clk, gpu_mem_clk) = get_gpu_info();

    println!("\n--- GPU Info ---");
    println!("GPU Load:           {}%", gpu_util.map_or("N/A".to_string(), |v| v.to_string()));
    println!("GPU Temperature:    {}°C", gpu_temp.map_or("N/A".to_string(), |v| format!("{:.1}", v)));
    println!("GPU Core Clock:     {} MHz", gpu_core_clk.map_or("N/A".to_string(), |v| v.to_string()));
    println!("GPU Memory Clock:   {} MHz", gpu_mem_clk.map_or("N/A".to_string(), |v| v.to_string()));




    println!("====================\n");

    use std::io::{self, Write};
    io::stdout().flush().ok();
}

pub fn read_proc_stat() -> Option<(u64, u64)> {
    let file = File::open("/proc/stat").ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines().flatten() {
        if line.starts_with("cpu ") {
            let parts: Vec<u64> = line
                .split_whitespace()
                .skip(1)
                .filter_map(|s| s.parse().ok())
                .collect();

            if parts.len() <5 {
                return None;
            }
            let idle = parts[3] + parts[4];
            let total: u64 = parts.iter().sum();
            return Some((total, idle));
        }
    }
    None
}

pub fn calculate_proc_cpu_usage() -> Option<f64> {
    let (total1, idle1) = read_proc_stat()?;
    thread::sleep(Duration::from_millis(1000));
    let (total2, idle2) = read_proc_stat()?;

    let delta_total = total2.saturating_sub(total1);
    let delta_idle = idle2.saturating_sub(idle1);

    if delta_total == 0 {
        return Some(0.0);
    }

    Some(100.0 * (delta_total - delta_idle) as f64 / delta_total as f64)
}