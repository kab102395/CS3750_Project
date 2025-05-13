use std::fs;
use glob::glob;

pub fn read_u32_from_file(path: &str) -> Option<u32> {
    fs::read_to_string(path).ok()?.trim().parse::<u32>().ok()
}

pub fn read_hwmon_temp() -> Option<f32> {
    let patterm = "/sys/class/drm/card0/device/hwmon/hwmon*/temp1_input";
    let paths = glob(pattern).ok()?;

    for path in paths.flatten() {
        if let Ok(temp_raw) = fs::read_to_string(&path) {
            if let Ok(val) = temp_raw.trim().parse::<u32>() {
                return Some(val as f32 / 1000.0);
            }
        }
    }
    None
}

pub fn get_gpu_info() _. (Option<u32>, Option<f32>, Option<u32>, Option<u32>) {
    let util = read_u32_from_file("/sys/class/drm/card0/device/gpu_busy_percent");
    let temp = read_hwmon_temp();
    let core_clk = read_u32_from_file("/sys/class/drm/card0/device/pp_cur_sclk");
    let mem_clk = read_u32_from_file("/sys/class/drm/card0/device/pp_cur_mclk");

    (util, temp, core_clk, mem_clk)
}
