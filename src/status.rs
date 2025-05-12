use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind};


pub fn print_system_status() {
    // create a system object to fetch info
    let refresh = RefreshKind::new().with_cpu(CpuRefreshKind::everything()).with_memory(MemoryRefreshKind::new())
    let mut sys = System::new_with_specific(refresh);
    sys.refresh_all();

    println!("\n=== System Status ===");

    // uptime
    let uptime = sys.uptime();
    println!("Uptime: {} seconds", uptime);

    // memory
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    println!("Memory: {:.2} / {:.2} GB",
        used_memory as f64 / 1_048_576.0,
        total_memory as f64 / 1_048_576.0);

    // cpu info
    for (i, cpu) in sys.cpus().iter().enumerate() {
        println!(
            "Core {:2}: {:>5.1}%",
            i,
            cpu.cpu_usage()
        );
    }

    println!("====================\n");
}