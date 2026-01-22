use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use gtk::{glib, prelude::*};
use gtk4 as gtk;
use hyprKCS::{cli, parser, ui};
use libadwaita as adw;

const APP_ID: &str = "com.github.hyprkcs";

fn main() -> glib::ExitCode {
    let args = cli::Args::parse();

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
                            let mods = b.mods.to_lowercase();
                            let key = b.key.to_lowercase();
                            let dispatcher = b.dispatcher.to_lowercase();

                            let args_str = b.args.to_lowercase();
                            let desc_str = b
                                .description
                                .as_ref()
                                .map(|s| s.to_lowercase())
                                .unwrap_or_default();

                            if let Some(ref q_mods) = query.mods {
                                if !mods.contains(q_mods) {
                                    return false;
                                }
                            }
                            if let Some(ref q_key) = query.key {
                                if !key.contains(q_key) {
                                    return false;
                                }
                            }
                            if let Some(ref q_action) = query.action {
                                if !dispatcher.contains(q_action) {
                                    return false;
                                }
                            }
                            if let Some(ref q_args) = query.args {
                                if !args_str.contains(q_args) {
                                    return false;
                                }
                            }
                            if let Some(ref q_desc) = query.description {
                                if !desc_str.contains(q_desc) {
                                    return false;
                                }
                            }

                            if query.general_query.is_empty() {
                                return true;
                            }
                            let text_to_match = &query.general_query;

                            matcher.fuzzy_match(&mods, text_to_match).is_some()
                                || matcher.fuzzy_match(&key, text_to_match).is_some()
                                || matcher.fuzzy_match(&dispatcher, text_to_match).is_some()
                                || matcher.fuzzy_match(&args_str, text_to_match).is_some()
                        })
                        .collect::<Vec<_>>()
                } else {
                    binds
                };

                // Simple manual table printing
                let mut w_mods = 9;
                let mut w_key = 3;
                let mut w_disp = 6;

                for b in &binds {
                    w_mods = w_mods.max(b.mods.len());
                    w_key = w_key.max(b.key.len());
                    w_disp = w_disp.max(b.dispatcher.len());
                }

                w_mods += 2;
                w_key += 2;
                w_disp += 2;

                println!(
                    "{:<w_mods$}{:<w_key$}{:<w_disp$}Arguments",
                    "Modifiers", "Key", "Action"
                );
                println!("{:-<100}", "");

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

    app.run_with_args(&Vec::<String>::new())
}
