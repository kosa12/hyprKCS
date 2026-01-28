use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudKeybind {
    pub mods: Box<str>,
    pub key: Box<str>,
    pub dispatcher: Box<str>,
    pub args: Box<str>,
}

impl HudKeybind {
    #[inline]
    pub fn new(mods: &str, key: &str, dispatcher: &str, args: &str) -> Self {
        Self {
            mods: mods.into(),
            key: key.into(),
            dispatcher: dispatcher.into(),
            args: args.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HudPosition {
    #[default]
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

impl HudPosition {
    pub fn from_str(s: &str) -> Self {
        match s {
            "top-left" => Self::TopLeft,
            "bottom-right" => Self::BottomRight,
            "bottom-left" => Self::BottomLeft,
            _ => Self::TopRight,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::TopRight => "top-right",
            Self::TopLeft => "top-left",
            Self::BottomRight => "bottom-right",
            Self::BottomLeft => "bottom-left",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HudConfig {
    pub enabled: bool,
    pub position: HudPosition,
    pub keybinds: Vec<HudKeybind>,
}

static HUD_CONFIG_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

#[inline]
pub fn get_hud_config_path() -> Option<&'static PathBuf> {
    HUD_CONFIG_PATH
        .get_or_init(|| {
            dirs::config_dir().map(|d| {
                d.join(super::constants::HYPRKCS_DIR)
                    .join(super::constants::HUD_CONF)
            })
        })
        .as_ref()
}

pub fn load_hud_config() -> HudConfig {
    let Some(path) = get_hud_config_path() else {
        return HudConfig::default();
    };

    let Ok(content) = fs::read_to_string(path) else {
        return HudConfig::default();
    };

    let mut config = HudConfig {
        enabled: false,
        position: HudPosition::TopRight,
        keybinds: Vec::with_capacity(8), // Pre-allocate for typical use
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(value) = line.strip_prefix("enabled=") {
            config.enabled = value == "true";
        } else if let Some(value) = line.strip_prefix("position=") {
            config.position = HudPosition::from_str(value);
        } else if line.contains('|') {
            let mut parts = line.splitn(4, '|');
            if let (Some(mods), Some(key), Some(disp), Some(args)) =
                (parts.next(), parts.next(), parts.next(), parts.next())
            {
                config.keybinds.push(HudKeybind::new(mods, key, disp, args));
            }
        }
    }
    config
}

pub fn save_hud_config(config: &HudConfig) -> std::io::Result<()> {
    let Some(path) = get_hud_config_path() else {
        return Ok(());
    };

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // Pre-calculate capacity: "enabled=true\n" + keybinds
    let capacity = 16
        + config
            .keybinds
            .iter()
            .map(|k| k.mods.len() + k.key.len() + k.dispatcher.len() + k.args.len() + 4)
            .sum::<usize>();

    let mut content = String::with_capacity(capacity);
    content.push_str("enabled=");
    content.push_str(if config.enabled { "true" } else { "false" });
    content.push('\n');
    content.push_str("position=");
    content.push_str(config.position.to_str());
    content.push('\n');

    for k in &config.keybinds {
        content.push_str(&k.mods);
        content.push('|');
        content.push_str(&k.key);
        content.push('|');
        content.push_str(&k.dispatcher);
        content.push('|');
        content.push_str(&k.args);
        content.push('\n');
    }

    let mut tmp_path = path.clone();
    tmp_path.set_extension("tmp");
    fs::write(&tmp_path, content)?;
    fs::rename(tmp_path, path)
}
