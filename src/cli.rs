use std::env;
use std::path::PathBuf;

pub struct Args {
    pub config: Option<PathBuf>,
    pub print: bool,
    pub search: Option<String>,
    pub doctor: bool,
}

impl Args {
    pub fn parse() -> Self {
        let mut config = None;
        let mut print = false;
        let mut search = None;
        let mut doctor = false;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-c" | "--config" => {
                    if let Some(path) = args.next() {
                        config = Some(PathBuf::from(path));
                    }
                }
                "-p" | "--print" => print = true,
                "-s" | "--search" => {
                    if let Some(term) = args.next() {
                        search = Some(term);
                    }
                }
                "--doctor" => doctor = true,
                "-h" | "--help" => {
                    println!("hyprKCS - Hyprland Keybind Cheat Sheet");
                    println!("\nUsage: hyprkcs [OPTIONS]");
                    println!("\nOptions:");
                    println!("  -c, --config <PATH>  Path to the Hyprland config file");
                    println!("  -p, --print          Print parsed keybinds to stdout and exit");
                    println!(
                        "  -s, --search <TERM>  Filter keybinds by a search term (implies --print)"
                    );
                    println!("  --doctor             Check system compatibility and report issues");
                    println!("  -h, --help           Print this help message");
                    std::process::exit(0);
                }
                "-v" | "--version" | "-V" => {
                    println!("hyprKCS v{}", env!("CARGO_PKG_VERSION"));
                    std::process::exit(0);
                }
                _ => {}
            }
        }

        Args {
            config,
            print,
            search,
            doctor,
        }
    }
}
