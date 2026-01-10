use clap::Parser;
use gtk4 as gtk;
use gtk::{glib, prelude::*};
use libadwaita as adw;

mod parser;
mod keybind_object;
mod ui;

const APP_ID: &str = "com.github.hyprkcs";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the Hyprland config file
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,

    /// Print parsed keybinds to stdout and exit
    #[arg(short, long)]
    print: bool,

    /// Filter keybinds by a search term (implies --print)
    #[arg(short, long)]
    search: Option<String>,
}

fn main() -> glib::ExitCode {
    let args = Args::parse();

    if let Some(config_path) = args.config {
        std::env::set_var("HYPRKCS_CONFIG", config_path);
    }

    if args.print || args.search.is_some() {
        match parser::parse_config() {
            Ok(binds) => {
                use comfy_table::presets::UTF8_FULL;
                use comfy_table::*;

                let binds = if let Some(term) = args.search {
                    let term = term.to_lowercase();
                    binds.into_iter().filter(|b| {
                        b.mods.to_lowercase().contains(&term) ||
                        b.key.to_lowercase().contains(&term) ||
                        b.dispatcher.to_lowercase().contains(&term) ||
                        b.args.to_lowercase().contains(&term)
                    }).collect::<Vec<_>>()
                } else {
                    binds
                };

                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec!["Modifiers", "Key", "Action", "Arguments"]);

                for bind in binds {
                    table.add_row(vec![
                        bind.mods,
                        bind.key,
                        bind.dispatcher,
                        bind.args,
                    ]);
                }

                println!("{table}");
            },
            Err(e) => eprintln!("Error parsing config: {}", e),
        }
        return glib::ExitCode::SUCCESS;
    }

    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| {
        adw::init().unwrap();
        let style_manager = adw::StyleManager::default();
        style_manager.set_color_scheme(adw::ColorScheme::Default);
        ui::style::load_css();
    });

    app.connect_activate(ui::window::build_ui);

    // We strip args so GTK doesn't complain about our custom flags
    app.run_with_args(&Vec::<String>::new())
}
