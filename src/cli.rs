use std::env;
use std::path::PathBuf;

pub struct Args {
    pub config: Option<PathBuf>,
    pub print: bool,
    pub search: Option<String>,
    pub doctor: bool,
    pub hud: bool,
}

impl Args {
    pub fn parse() -> Self {
        Self::parse_from(std::env::args())
    }

    pub fn parse_from<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let mut config = None;
        let mut print = false;
        let mut search = None;
        let mut doctor = false;
        let mut hud = false;

        let mut args_iter = args.into_iter().skip(1);
        while let Some(arg) = args_iter.next() {
            let arg_str = arg.into();
            match arg_str.as_str() {
                "-c" | "--config" => {
                    if let Some(path) = args_iter.next() {
                        config = Some(PathBuf::from(path.into()));
                    }
                }
                "-p" | "--print" => print = true,
                "-s" | "--search" => {
                    if let Some(term) = args_iter.next() {
                        search = Some(term.into());
                    }
                }
                "--doctor" => doctor = true,
                "--hud" => hud = true,
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
                    println!("  --hud                Launch the Wallpaper HUD");
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
            hud,
        }
    }
}
