use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use gtk::{glib, prelude::*};
use gtk4 as gtk;
use hyprKCS::{cli, parser, ui};
use libadwaita as adw;

const APP_ID: &str = "com.github.hyprkcs";

fn main() -> glib::ExitCode {
    let args = cli::Args::parse();

    if args.doctor {
        hyprKCS::doctor::run_doctor();
        return glib::ExitCode::SUCCESS;
    }

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
                                if !mods.contains(q_mods.as_ref()) {
                                    return false;
                                }
                            }
                            if let Some(ref q_key) = query.key {
                                if !key.contains(q_key.as_ref()) {
                                    return false;
                                }
                            }
                            if let Some(ref q_action) = query.action {
                                if !dispatcher.contains(q_action.as_ref()) {
                                    return false;
                                }
                            }
                            if let Some(ref q_args) = query.args {
                                if !args_str.contains(q_args.as_ref()) {
                                    return false;
                                }
                            }
                            if let Some(ref q_desc) = query.description {
                                if !desc_str.contains(q_desc.as_ref()) {
                                    return false;
                                }
                            }

                            if query.general_query.is_empty() {
                                return true;
                            }
                            let text_to_match: &str = query.general_query.as_ref();

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

    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        adw::init().unwrap();
        let style_manager = adw::StyleManager::default();
        style_manager.set_color_scheme(adw::ColorScheme::Default);
        ui::style::load_css();
    });

    app.connect_activate(ui::window::build_ui);

    // Cleanup on shutdown
    app.connect_shutdown(|_| {
        ui::style::cleanup();
    });

    app.run_with_args(&Vec::<String>::new())
}
