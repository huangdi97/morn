pub fn setup_autostart(app: &tauri::App) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let autostart_dir = std::path::PathBuf::from(&home).join(".config/autostart");
            if std::fs::create_dir_all(&autostart_dir).is_ok() {
                if let Ok(exe) = std::env::current_exe() {
                    let desktop_entry = format!(
                        "[Desktop Entry]\nType=Application\nName=Morn\nExec={}\nX-GNOME-Autostart-enabled=true\n",
                        exe.display()
                    );
                    let _ =
                        std::fs::write(autostart_dir.join("morn-desktop.desktop"), desktop_entry);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new("reg")
                .args([
                    "add",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "/v",
                    "Morn",
                    "/d",
                    &exe.display().to_string(),
                    "/f",
                ])
                .output();
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let (Ok(home), Ok(exe)) = (std::env::var("HOME"), std::env::current_exe()) {
            let launch_agents = std::path::PathBuf::from(home)
                .join("Library")
                .join("LaunchAgents");
            if std::fs::create_dir_all(&launch_agents).is_ok() {
                let plist = format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>ai.morn.desktop</string>
  <key>ProgramArguments</key>
  <array>
    <string>{}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
</dict>
</plist>
"#,
                    exe.display()
                );
                let _ = std::fs::write(launch_agents.join("ai.morn.desktop.plist"), plist);
            }
        }
    }
}
