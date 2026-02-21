
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_installed_apps() {
        let apps = get_installed_apps();
        println!("Found {} apps", apps.len());
        for app in apps.iter().take(20) {
            println!("Name: {}, Exec: {}", app.name, app.exec);
        }
        
        let firefox = apps.iter().find(|a| a.name.to_lowercase().contains("firefox"));
        assert!(firefox.is_some(), "Firefox should be found if installed");
    }
}
