use clap::Parser;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use gtk::{glib, prelude::*};
use gtk4 as gtk;
use libadwaita as adw;

mod config;
mod keybind_object;
mod parser;
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
                let binds = if let Some(term) = args.search {
                    let matcher = SkimMatcherV2::default();
                    let query = ui::utils::SearchQuery::parse(&term);

                    binds
                        .into_iter()
                        .filter(|b| {
                            let mods = &b.mods;
                            let key = &b.key;
                            let dispatcher = b.dispatcher.to_lowercase();
                            let args = b.args.to_lowercase();
                            let description = b.description.clone().unwrap_or_default().to_lowercase();

                            if let Some(ref q_mods) = query.mods {
                                if !mods.to_lowercase().contains(&q_mods.to_lowercase()) {
                                    return false;
                                }
                            }
                            if let Some(ref q_key) = query.key {
                                if !key.to_lowercase().contains(&q_key.to_lowercase()) {
                                    return false;
                                }
                            }
                            if let Some(ref q_action) = query.action {
                                if !dispatcher.contains(&q_action.to_lowercase()) {
                                    return false;
                                }
                            }
                            if let Some(ref q_args) = query.args {
                                if !args.contains(&q_args.to_lowercase()) {
                                    return false;
                                }
                            }
                            if let Some(ref q_desc) = query.description {
                                if !description.contains(&q_desc.to_lowercase()) {
                                    return false;
                                }
                            }

                            if query.general_query.is_empty() {
                                return true;
                            }
                            let text_to_match = &query.general_query;

                            matcher.fuzzy_match(mods, text_to_match).is_some()
                                || matcher.fuzzy_match(key, text_to_match).is_some()
                                || matcher.fuzzy_match(&dispatcher, text_to_match).is_some()
                                || matcher.fuzzy_match(&args, text_to_match).is_some()
                        })
                        .collect::<Vec<_>>()
                } else {
                    binds
                };

                // Simple manual table printing
                // Calculate max widths for alignment
                let mut w_mods = 9; // "Modifiers".len()
                let mut w_key = 3; // "Key".len()
                let mut w_disp = 6; // "Action".len()

                for b in &binds {
                    w_mods = w_mods.max(b.mods.len());
                    w_key = w_key.max(b.key.len());
                    w_disp = w_disp.max(b.dispatcher.len());
                }

                // Add padding
                w_mods += 2;
                w_key += 2;
                w_disp += 2;

                // Header
                println!(
                    "{:<w_mods$}{:<w_key$}{:<w_disp$}Arguments",
                    "Modifiers", "Key", "Action"
                );
                println!("{:-<100}", ""); // Separator

                for bind in binds {
                    println!(
                        "{:<w_mods$}{:<w_key$}{:<w_disp$}{}",
                        bind.mods, bind.key, bind.dispatcher, bind.args
                    );
                }
            }
            Err(e) => eprintln!("Error parsing config: {}", e),
        }
        return glib::ExitCode::SUCCESS;
    }

    let app = adw::Application::builder().application_id(APP_ID).build();

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
