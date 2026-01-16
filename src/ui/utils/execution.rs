pub fn execute_keybind(dispatcher: &str, args: &str) {
    let variables = crate::parser::get_variables().unwrap_or_default();

    let mut resolved_dispatcher = dispatcher.to_string();
    let mut resolved_args = args.to_string();

    let mut sorted_vars: Vec<_> = variables.keys().collect();
    sorted_vars.sort_by(|a, b| b.len().cmp(&a.len()));

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
    let cmd_name = if let Some(first_part) = command.split_whitespace().next() {
        first_part
    } else {
        return false;
    };

    if std::path::Path::new(cmd_name).is_absolute() {
        return std::path::Path::new(cmd_name).exists();
    }

    if let Ok(path) = std::env::var("PATH") {
        for p in std::env::split_paths(&path) {
            let full_path = p.join(cmd_name);
            if full_path.is_file() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = std::fs::metadata(&full_path) {
                        if metadata.permissions().mode() & 0o111 != 0 {
                            return true;
                        }
                    }
                }
                #[cfg(not(unix))]
                return true;
            }
        }
    }
    false
}
