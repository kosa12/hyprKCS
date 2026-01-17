use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FavoriteKeybind {
    pub mods: String,
    pub key: String,
    pub submap: String,
    pub dispatcher: String,
    pub args: String,
}

pub fn get_favorites_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| {
        d.join(super::constants::HYPRKCS_DIR)
            .join(super::constants::FAVORITES_JSON)
    })
}

pub fn load_favorites() -> Vec<FavoriteKeybind> {
    if let Some(path) = get_favorites_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(favs) = serde_json::from_str(&content) {
                    return favs;
                }
            }
        }
    }
    Vec::new()
}

pub fn save_favorites(favorites: &[FavoriteKeybind]) -> std::io::Result<()> {
    if let Some(path) = get_favorites_path() {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let content = serde_json::to_string_pretty(favorites)?;
        fs::write(path, content)?;
    }
    Ok(())
}

pub fn is_favorite(
    favs: &[FavoriteKeybind],
    mods: &str,
    key: &str,
    submap: &str,
    dispatcher: &str,
    args: &str,
) -> bool {
    favs.iter().any(|f| {
        f.mods == mods
            && f.key == key
            && f.submap == submap
            && f.dispatcher == dispatcher
            && f.args == args
    })
}

pub fn toggle_favorite(favs: &mut Vec<FavoriteKeybind>, item: FavoriteKeybind) -> bool {
    if let Some(pos) = favs.iter().position(|f| *f == item) {
        favs.remove(pos);
        false
    } else {
        favs.push(item);
        true
    }
}
