pub fn execute_keybind(dispatcher: &str, args: &str) {
    let variables = crate::parser::get_variables().unwrap_or_default();

    let mut resolved_dispatcher = dispatcher.to_string();
    let mut resolved_args = args.to_string();

    let mut sorted_vars: Vec<_> = variables.keys().collect();
    sorted_vars.sort_by_key(|b: &&String| std::cmp::Reverse(b.len()));

    for key in sorted_vars {
        if resolved_dispatcher.contains(key) {
            resolved_dispatcher = resolved_dispatcher.replace(key, &variables[key]);
        }
        if resolved_args.contains(key) {
            resolved_args = resolved_args.replace(key, &variables[key]);
        }
    }

    let mut command = std::process::Command::new("hyprctl");
    command.arg("dispatch").arg(&resolved_dispatcher);

    if !resolved_args.trim().is_empty() {
        command.arg(&resolved_args);
    }

    let _ = command.spawn();
}

pub fn execute_hyprctl(args: &[&str]) {
    use std::io::Write;

    let output = std::process::Command::new("hyprctl").args(args).output();

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/hyprkcs-debug.log")
    {
        let _ = writeln!(file, "Executing: hyprctl {:?}", args);
        match &output {
            Ok(out) => {
                let _ = writeln!(file, "Status: {}", out.status);
                let _ = writeln!(file, "Stdout: {}", String::from_utf8_lossy(&out.stdout));
                let _ = writeln!(file, "Stderr: {}", String::from_utf8_lossy(&out.stderr));
            }
            Err(e) => {
                let _ = writeln!(file, "Failed to execute: {}", e);
            }
        }
    }

    if let Err(e) = output {
        eprintln!("Failed to execute hyprctl: {}", e);
    }
}

pub fn command_exists(command: &str) -> bool {
    let mut cmd = command.trim();

    // Strip Hyprland exec flags like [float] or [workspace 1]
    if cmd.starts_with('[') {
        if let Some(end_idx) = cmd.find(']') {
            cmd = cmd[end_idx + 1..].trim();
        }
    }

    let cmd_name = if let Some(first_part) = cmd.split_whitespace().next() {
        first_part
    } else {
        return false;
    };

    // Handle home directory expansion
    let path_to_check = if cmd_name.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            home.join(&cmd_name[2..])
        } else {
            std::path::PathBuf::from(cmd_name)
        }
    } else {
        std::path::PathBuf::from(cmd_name)
    };

    let exists = if path_to_check.is_absolute() {
        path_to_check.exists()
    } else if let Ok(path) = std::env::var("PATH") {
        let mut found = false;
        for p in std::env::split_paths(&path) {
            let full_path = p.join(cmd_name);
            if full_path.is_file() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = std::fs::metadata(&full_path) {
                        if metadata.permissions().mode() & 0o111 != 0 {
                            found = true;
                            break;
                        }
                    }
                }
                #[cfg(not(unix))]
                {
                    found = true;
                    break;
                }
            }
        }
        found
    } else {
        false
    };

    exists
}
