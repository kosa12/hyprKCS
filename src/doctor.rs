use crate::parser::{get_config_path, parse_config};
use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;
use std::env;
use std::fs;
use std::process::Command;

pub fn run_doctor() {
    println!("hyprKCS Doctor Report");
    println!("=====================\n");

    let pass = "[\x1b[32mPASS\x1b[0m]";
    let fail = "[\x1b[31mFAIL\x1b[0m]";
    let warn = "[\x1b[33mWARN\x1b[0m]";
    let info = "[\x1b[34mINFO\x1b[0m]";

    println!("1. System Information");
    println!("---------------------");

    let os_release = fs::read_to_string("/etc/os-release").unwrap_or_default();
    let pretty_name = os_release
        .lines()
        .find(|l| l.starts_with("PRETTY_NAME="))
        .map(|l| l.trim_start_matches("PRETTY_NAME=").trim_matches('"'))
        .unwrap_or("Linux (Unknown)");
    println!("{} OS: {}", info, pretty_name);

    let session = env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".into());
    if session == "wayland" {
        println!("{} Session Type: Wayland", pass);
    } else {
        println!(
            "{} Session Type: {} (GTK4 requires Wayland for Layer Shell features)",
            warn, session
        );
    }

    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        println!("{} Hyprland Instance: Detected", pass);
    } else {
        println!(
            "{} Hyprland Instance: Not detected (Is Hyprland running?)",
            fail
        );
    }
    println!();

    println!("2. Dependencies & Runtime");
    println!("-----------------------");

    let major = gtk::major_version();
    let minor = gtk::minor_version();
    let micro = gtk::micro_version();
    let ver_str = format!("{}.{}.{}", major, minor, micro);

    if major >= 4 && minor >= 10 {
        println!("{} GTK4 Version: {}", pass, ver_str);
    } else {
        println!("{} GTK4 Version: {} (Recommended: 4.10+)", warn, ver_str);
    }

    glib::log_set_writer_func(|level, fields| {
        for field in fields {
            if field.key() == "MESSAGE" {
                if let Some(msg) = field.value_str() {
                    if msg.contains("gtk-application-prefer-dark-theme") {
                        return glib::LogWriterOutput::Handled;
                    }
                }
            }
        }
        glib::log_writer_default(level, fields)
    });

    match adw::init() {
        Ok(_) => println!("{} Libadwaita: Initialized successfully", pass),
        Err(_) => println!(
            "{} Libadwaita: Initialization failed (Display server issue?)",
            fail
        ),
    }

    match Command::new("hyprctl").arg("version").output() {
        Ok(out) => {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout);
                let tag = s
                    .lines()
                    .find(|l| l.contains("Tag:"))
                    .map(|l| l.trim())
                    .unwrap_or("Unknown Tag");
                println!("{} Hyprland CLI: Reachable ({})", pass, tag);
            } else {
                println!("{} Hyprland CLI: Error executing command", fail);
            }
        }
        Err(_) => println!("{} Hyprland CLI: 'hyprctl' not found in PATH", fail),
    }
    println!();

    println!("3. Configuration Access");
    println!("-----------------------");

    match get_config_path() {
        Ok(path) => {
            if path.exists() {
                println!("{} Config File: Found at {:?}", pass, path);

                match fs::metadata(&path) {
                    Ok(meta) => {
                        if meta.permissions().readonly() {
                            println!("{} Permissions: Read-only (Saving will fail)", fail);
                        } else {
                            println!("{} Permissions: Writable", pass);
                        }
                    }
                    Err(e) => println!("{} Permissions: Check failed ({})", warn, e),
                }

                match parse_config() {
                    Ok(binds) => {
                        println!(
                            "{} Parser: Successfully parsed {} keybinds",
                            pass,
                            binds.len()
                        );
                    }
                    Err(e) => println!("{} Parser: Failed to parse config ({})", fail, e),
                }
            } else {
                println!("{} Config File: Not found at {:?}", fail, path);
            }
        }
        Err(e) => println!("{} Config File: Discovery failed ({})", fail, e),
    }
    println!();

    println!("4. System Environment");
    println!("---------------------");

    let lang = env::var("LANG").unwrap_or_else(|_| "Unset".into());
    println!("{} Locale (LANG): {}", info, lang);

    match Command::new("hyprctl").args(["-j", "devices"]).output() {
        Ok(out) => {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout);
                let kbd_count = s.matches("\"keyboards\":").count();
                if kbd_count > 0 {
                    println!("{} Input Devices: Hyprland detected keyboards", pass);

                    let mut layout_code = String::new();
                    let mut keymap_name = String::new();

                    if let Some(idx) = s.find("\"layout\":") {
                        let rest = &s[idx..];
                        if let Some(start) = rest.find(':') {
                            if let Some(end) = rest[start..].find(',') {
                                layout_code = rest[start + 1..start + end]
                                    .trim()
                                    .trim_matches('"')
                                    .to_string();
                            }
                        }
                    }

                    if let Some(idx) = s.find("\"active_keymap\":") {
                        let rest = &s[idx..];
                        if let Some(start) = rest.find(':') {
                            if let Some(end) = rest[start..].find('\n') {
                                let val_part = &rest[start + 1..start + end];
                                keymap_name = val_part
                                    .trim()
                                    .trim_matches('"')
                                    .trim_matches(',')
                                    .trim_matches('"')
                                    .to_string();
                            }
                        }
                    }

                    if !keymap_name.is_empty() {
                        println!("{} Keyboard Layout: {}", info, keymap_name);
                    }

                    if !layout_code.is_empty() {
                        let physical_guess = match layout_code.as_str() {
                            "us" => "ANSI (likely)",
                            "jp" => "JIS (likely)",
                            "br" => "ABNT2 (likely)",
                            "hu" | "de" | "gb" | "fr" | "it" | "es" | "pt" | "se" | "no" | "dk"
                            | "fi" => "ISO (likely)",
                            _ => "Unknown (shape cannot be determined from code)",
                        };
                        println!(
                            "{} Physical Shape: {} [Code: {}]",
                            info, physical_guess, layout_code
                        );
                    }
                } else {
                    println!(
                        "{} Input Devices: No keyboards section found (Old Hyprland?)",
                        warn
                    );
                }
            } else {
                println!("{} Input Devices: Failed to query (IPC Error)", fail);
            }
        }
        Err(_) => println!("{} Input Devices: Skipped (hyprctl not found)", warn),
    }

    if let Some(config_dir) = dirs::config_dir() {
        let backup_dir = config_dir.join("hypr/backups");
        if backup_dir.exists() {
            match fs::metadata(&backup_dir) {
                Ok(meta) => {
                    if meta.permissions().readonly() {
                        println!("{} Backups: Directory exists but is Read-Only", fail);
                    } else {
                        println!("{} Backups: Directory writable ({:?})", pass, backup_dir);
                    }
                }
                Err(_) => println!(
                    "{} Backups: Directory exists (Permission check failed)",
                    warn
                ),
            }
        } else {
            println!(
                "{} Backups: Directory does not exist yet (Will be created)",
                info
            );
        }
    }

    println!();

    println!("Doctor check complete.");
}
