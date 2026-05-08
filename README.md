# Steam Deck Optimizer

A native Linux performance utility for the Steam Deck, written in Rust. Built as the capstone project for CS3750 (Software Engineering) at CSU Stanislaus.

The tool gives you real-time system monitoring, CPU governor management, GPU diagnostics, and automated game detection — all from a single binary that runs as either a full egui GUI application or a headless CLI tool, depending on how you invoke it.

---

## Features

### Performance Mode Management
Switch between preset CPU governor profiles with one click or one command:

| Mode | Governor Priority | Use Case |
|---|---|---|
| Battery Saver | `powersave` → `schedutil` | Maximum battery life |
| Balanced | `ondemand` → `schedutil` | Everyday use |
| Performance | `performance` | Demanding games |
| Custom | Any governor on your kernel | Direct control |

Modes are resolved at runtime by checking `/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors` — no assumptions made about what your kernel supports. Reset falls back to the best available governor automatically.

### System Status & Monitoring
- **Accurate CPU usage** read directly from `/proc/stat` using a two-sample delta — not sysinfo polling, which is unreliable for a single sample
- **Per-core CPU breakdown** via `sysinfo`
- **RAM usage** in GB (used / total)
- **System uptime**
- **GPU diagnostics** — load %, temperature, core clock (MHz), memory clock (MHz)

### AMD GPU Diagnostics
Reads from multiple sources with automatic fallback priority:

1. `/sys/kernel/debug/dri/*/amdgpu_pm_info` — GPU/VRAM load percentage and clock speeds (debugfs, with `sudo cat` fallback on permission denial)
2. HWMon sysfs (`temp1_input`, `fan1_input`, `in0_input`, `power1_average`) — temperature, fan RPM, voltage, and power draw in watts
3. Direct sysfs mem_info files — VRAM, visible VRAM, and GTT usage/total in bytes
4. `gpu_busy_percent` / `mem_busy_percent` sysfs fallback if debugfs is unavailable

### JSON System Logging
Captures a full system snapshot to `logs/system_log_<unix_timestamp>.json`. The GUI's "Show System Status" button reads and displays the most recent log inline using an async `mpsc` channel so the UI never blocks.

Log schema:
```json
{
  "timestamp": 1714512000,
  "uptime": 3600,
  "memory_used_gb": 6.42,
  "memory_total_gb": 15.9,
  "accurate_cpu_total": 34.2,
  "sysinfo_cpu_total": 33.1,
  "per_core": [12.0, 45.0, 28.0, 50.0],
  "gpu_util_percent": 72,
  "gpu_temp_celsius": 68.5,
  "gpu_core_clock_mhz": 1600,
  "gpu_mem_clock_mhz": 800
}
```

### Game Detection
- **Steam** — parses `steamapps/libraryfolders.vdf` to discover all library paths, scans each `common/` directory, pulls cover art from the Steam grid cache (`userdata/<id>/config/grid/`)
- **Prism Launcher** — discovers all Minecraft instances under `~/.local/share/PrismLauncher/instances` with instance icons
- Game cards render in the GUI with 64×64 cover images loaded via `egui_extras`

---

## Architecture

```
src/
├── main.rs         — Entry point; routes to GUI or CLI based on flag presence
├── cli.rs          — clap argument definitions (--status, --mode, --reset, --log)
├── gui.rs          — egui/eframe GUI; async status loading via mpsc channel
├── modes.rs        — CPU governor logic; mode enum, apply, reset, get_available
├── status.rs       — /proc/stat CPU measurement; sysinfo memory + per-core
├── hardware.rs     — AMD GPU sysfs + debugfs reader; full AMDGPUStats struct
├── logger.rs       — JSON log writer and latest-log reader
├── games.rs        — Steam + Prism Launcher game discovery
└── permissions.rs  — udev rules, video group membership, debugfs mount check
```

**Dual-mode dispatch:** If no CLI flags are passed, the binary launches the GUI. If any flag is present, it runs headlessly and exits. The same binary works as a desktop app or a scriptable system tool.

---

## Usage

### GUI (default)
```bash
cargo run
```

### CLI
```bash
# Show system status
cargo run -- --status

# Apply a performance mode
cargo run -- --mode gaming
cargo run -- --mode balance
cargo run -- --mode battery

# Log current stats to JSON
cargo run -- --log

# Reset CPU governor to system default
cargo run -- --reset
```

---

## Building

```bash
git clone https://github.com/YOUR_USERNAME/CS3750_Project
cd CS3750_Project
cargo build --release
./target/release/CS3750_Project
```

**Requirements:**
- Linux (Steam Deck / SteamOS, Arch, or any systemd-based distro)
- AMD GPU (tested on Steam Deck APU)
- The binary uses `sudo tee` to write governor files — you will be prompted on first run if permissions are not yet set up

---

## Dependencies

| Crate | Purpose |
|---|---|
| `clap 4.4` | CLI argument parsing |
| `sysinfo 0.30` | CPU/memory polling |
| `serde` + `serde_json` | Log serialization |
| `eframe` + `egui 0.27` | Native GUI |
| `egui_extras 0.27` | Image loading for game covers |
| `glob 0.3` | debugfs path discovery |
| `nix 0.27` | GID checks for permission validation |

---

## GPU Access & Permissions

On first run, `permissions.rs` checks and handles:

- **video group membership** — adds your user via `usermod -aG video` if missing
- **udev rules** — writes `/etc/udev/rules.d/99-gpu-permissions.rules` for `card*` and `renderD*` device access
- **debugfs mount** — mounts `/sys/kernel/debug` if not already present
- **amdgpu_pm_info permissions** — fixes group/mode on the debugfs file if your GID doesn't match

These are one-time setup steps. After a reboot or re-login the tool runs without elevated prompts for normal operation.

---

## Project Context

Built for CS3750 — Software Engineering at CSU Stanislaus. The goal was to build something actually useful rather than contrived, targeting a real use case: Steam Deck users who want hardware-level control without dropping into a terminal every session.

The codebase demonstrates multi-source hardware telemetry with graceful fallback, async UI patterns in an immediate-mode GUI, dual CLI/GUI dispatch from a single binary, and Linux sysfs/debugfs integration written in safe Rust.

---

## Author

Kyle Anthony Barrett — [Ember Tech Solutions LLC](https://github.com/YOUR_USERNAME)
