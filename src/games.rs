use std::path::{PathBuf};
use std::fs;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub name: String,
    pub exe_path: PathBuf,
    pub cover_image: Option<PathBuf>,
    pub source: String, // "Steam" or "Prism"
}

pub fn discover_steam_games() -> Vec<GameInfo> {
    let mut games = vec![];
    if let Some(steam_root) = find_steam_root() {
        let library_paths = parse_libraryfolders(&steam_root);
        for lib_path in library_paths {
            let common_path = lib_path.join("steamapps/common");
            if let Ok(entries) = fs::read_dir(&common_path) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let exe_path = entry.path();
                            let art = get_cover_for_steam_game(&steam_root, &name);
                            games.push(GameInfo {
                                name,
                                exe_path,
                                cover_image: art,
                                source: "Steam".into(),
                            });
                        }
                    }
                }
            }
        }
    }
    games
}

fn find_steam_root() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let path = PathBuf::from(format!("{}/.steam/steam", home));
    if path.exists() { Some(path) } else { None }
}

fn parse_libraryfolders(steam_root: &PathBuf) -> Vec<PathBuf> {
    let mut paths = vec![steam_root.clone()];
    let config_path = steam_root.join("steamapps/libraryfolders.vdf");
    if let Ok(content) = fs::read_to_string(config_path) {
        for line in content.lines() {
            if line.contains("\"path\"") {
                if let Some(path_str) = line.split('"').nth(3) {
                    let path = PathBuf::from(path_str);
                    if path.exists() {
                        paths.push(path);
                    }
                }
            }
        }
    }
    paths
}

fn get_cover_for_steam_game(steam_root: &PathBuf, game_name: &str) -> Option<PathBuf> {
    let grid_path = steam_root.join("userdata");
    if let Ok(users) = fs::read_dir(grid_path) {
        for user in users.flatten() {
            let grid_dir = user.path().join("config/grid");
            if grid_dir.exists() {
                for ext in ["png", "jpg", "jpeg"] {
                    let cover = grid_dir.join(format!("{}.{}", game_name, ext));
                    if cover.exists() {
                        return Some(cover);
                    }
                }
            }
        }
    }
    None
}

pub fn discover_prism_games() -> Vec<GameInfo> {
    let mut games = vec![];
    let home = std::env::var("HOME").unwrap_or_default();
    let prism_path = PathBuf::from(format!("{}/.local/share/PrismLauncher/instances", home));
    if let Ok(entries) = fs::read_dir(&prism_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let exe_path = entry.path().join(".minecraft/launcher_profiles.json");
            let icon = entry.path().join("icon.png");

            games.push(GameInfo {
                name,
                exe_path,
                cover_image: if icon.exists() { Some(icon) } else { None },
                source: "Prism".into(),
            });
        }
    }
    games
}

pub fn discover_all_games() -> Vec<GameInfo> {
    let mut all = discover_steam_games();
    all.extend(discover_prism_games());
    all
}
