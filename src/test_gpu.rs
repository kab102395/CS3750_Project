use std::fs;

fn main() {
    let path = "/sys/kernel/debug/dri/0000:c6:00.0/amdgpu_pm_info";
    match fs::read_to_string(path) {
        Ok(content) => {
            println!("--- Raw Output ---");
            println!("{}", content);

            println!("\n--- Filtered Output ---");
            for line in content.lines() {
                if line.contains("GPU Load")
                    || line.contains("GPU Temperature")
                    || line.contains("MCLK")
                    || line.contains("SCLK")
                {
                    println!("{}", line);
                }
            }
        }
        Err(e) => println!("Error reading file: {}", e),
    }
}
