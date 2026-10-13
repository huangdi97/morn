//! Morn desktop boundary shell.
//!
//! The desktop process never owns business truth. The unsigned/reference
//! package may own the lifecycle of its sibling `server(.exe)` so users get
//! one-launch local UX while all Work/domain logic remains in `morn-app`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::Manager;

const DEFAULT_API_BASE: &str = "http://127.0.0.1:8090";
const DEFAULT_BACKEND_ADDR: &str = "127.0.0.1:8090";
const DESKTOP_INSTANCE_LOCK_ADDR: &str = "127.0.0.1:18091";

#[derive(Debug, Default)]
struct OwnedBackend {
    child: Option<Child>,
}

impl OwnedBackend {
    fn stop(&mut self) -> bool {
        let Some(mut child) = self.child.take() else {
            return true;
        };
        // The owned server watches stdin EOF. Closing only our pipe asks for
        // graceful Axum/SQLite shutdown without exposing a management endpoint.
        drop(child.stdin.take());
        let deadline = Instant::now() + Duration::from_secs(3);
        let graceful = loop {
            match child.try_wait() {
                Ok(Some(_)) => break true,
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                _ => {
                    let _ = child.kill();
                    break false;
                }
            }
        };
        let _ = child.wait();
        graceful
    }
}

impl Drop for OwnedBackend {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn autostart_enabled() -> bool {
    !std::env::var("MORN_DESKTOP_AUTOSTART_BACKEND")
        .ok()
        .is_some_and(|value| matches!(value.trim(), "0" | "false" | "FALSE" | "no" | "NO"))
}

fn sibling_backend_path(current_exe: &Path) -> Option<PathBuf> {
    let parent = current_exe.parent()?;
    #[cfg(target_os = "windows")]
    let name = "server.exe";
    #[cfg(not(target_os = "windows"))]
    let name = "server";
    Some(parent.join(name))
}

fn backend_addr() -> SocketAddr {
    DEFAULT_BACKEND_ADDR
        .parse()
        .expect("static Morn backend address is valid")
}

fn backend_port_open() -> bool {
    TcpStream::connect_timeout(&backend_addr(), Duration::from_millis(150)).is_ok()
}

fn backend_is_healthy() -> bool {
    let Ok(mut stream) = TcpStream::connect_timeout(&backend_addr(), Duration::from_millis(250))
    else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(750)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
    if stream
        .write_all(b"GET /api/health HTTP/1.1\r\nHost: 127.0.0.1:8090\r\nConnection: close\r\n\r\n")
        .is_err()
    {
        return false;
    }
    let mut response = String::new();
    if stream.read_to_string(&mut response).is_err() {
        return false;
    }
    response.starts_with("HTTP/1.1 200")
        && response.contains("\"status\":\"ok\"")
        && response.contains("\"service\":\"morn-app\"")
}

fn acquire_instance_lock() -> Result<TcpListener, String> {
    TcpListener::bind(DESKTOP_INSTANCE_LOCK_ADDR).map_err(|error| {
        format!(
            "another Morn desktop instance is already active or the local instance-lock address {DESKTOP_INSTANCE_LOCK_ADDR} is unavailable: {error}"
        )
    })
}

fn wait_for_backend(child: &mut Child, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    loop {
        if backend_is_healthy() {
            return Ok(());
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("check Morn backend process: {error}"))?
        {
            return Err(format!(
                "Morn backend exited before becoming healthy: {status}"
            ));
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "Morn backend did not become healthy at {DEFAULT_API_BASE} within {} ms",
                timeout.as_millis()
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn spawn_reference_backend(data_dir: &Path) -> Result<Option<Child>, String> {
    if !autostart_enabled() {
        return Ok(None);
    }
    if backend_is_healthy() {
        // Reuse a real, already-running Morn backend, but never claim ownership.
        return Ok(None);
    }
    if backend_port_open() {
        return Err(format!(
            "{DEFAULT_BACKEND_ADDR} is occupied by a service that does not pass Morn /api/health; refusing to attach or kill it"
        ));
    }

    let current_exe =
        std::env::current_exe().map_err(|error| format!("resolve desktop executable: {error}"))?;
    let backend = sibling_backend_path(&current_exe)
        .ok_or_else(|| "desktop executable has no parent directory".to_string())?;
    if !backend.is_file() {
        return Err(format!(
            "local Morn backend not found beside desktop executable: {}.              Build/package server(.exe), start a healthy local backend yourself,              or set MORN_DESKTOP_AUTOSTART_BACKEND=0.",
            backend.display()
        ));
    }

    std::fs::create_dir_all(data_dir)
        .map_err(|error| format!("create Morn local data directory: {error}"))?;
    let db_path = std::env::var_os("MORN_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir.join("morn.db"));

    let mut child = Command::new(&backend)
        .current_dir(data_dir)
        .env("MORN_PORT", "8090")
        .env("MORN_DB", db_path)
        .env("MORN_EXIT_ON_STDIN_CLOSE", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("start local Morn backend {}: {error}", backend.display()))?;

    if let Err(error) = wait_for_backend(&mut child, Duration::from_secs(8)) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    Ok(Some(child))
}

fn run_sidecar_probe() -> Result<(), String> {
    let data_dir =
        std::env::temp_dir().join(format!("morn-desktop-sidecar-probe-{}", std::process::id()));
    let mut owned = OwnedBackend {
        child: spawn_reference_backend(&data_dir)?,
    };
    if !backend_is_healthy() {
        owned.stop();
        let _ = std::fs::remove_dir_all(&data_dir);
        return Err("desktop sidecar probe never observed a healthy Morn backend".to_string());
    }
    if !owned.stop() {
        let _ = std::fs::remove_dir_all(&data_dir);
        return Err("desktop sidecar required force-kill instead of graceful stdin-EOF shutdown".to_string());
    }
    // A child owned by this probe must actually release the listening socket.
    let deadline = Instant::now() + Duration::from_secs(4);
    while backend_port_open() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    if backend_port_open() {
        let _ = std::fs::remove_dir_all(&data_dir);
        return Err("desktop sidecar probe did not release the backend port".to_string());
    }
    let _ = std::fs::remove_dir_all(&data_dir);
    Ok(())
}

/// Report the reference desktop shell API boundary.
#[tauri::command]
fn app_info() -> serde_json::Value {
    serde_json::json!({
        "shell": "morn-desktop",
        "version": env!("CARGO_PKG_VERSION"),
        "api_base": DEFAULT_API_BASE,
        "backend_autostart": autostart_enabled(),
    })
}

fn main() {
    if std::env::args().any(|arg| arg == "--probe-sidecar") {
        if let Err(error) = run_sidecar_probe() {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }

    // The reference desktop owns at most one local backend. A second desktop
    // cannot safely share that owned child because the first owner must reap it
    // on exit. Hold a loopback listener for the full GUI process lifetime.
    let _instance_lock = acquire_instance_lock().unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });

    let backend = Arc::new(Mutex::new(OwnedBackend::default()));
    let setup_backend = Arc::clone(&backend);
    let exit_backend = Arc::clone(&backend);

    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_info])
        .setup(move |app| {
            let data_dir = app.path().app_local_data_dir().map_err(|error| {
                std::io::Error::other(format!("desktop data directory: {error}"))
            })?;
            let child = spawn_reference_backend(&data_dir).map_err(|error| {
                std::io::Error::other(format!("desktop backend setup: {error}"))
            })?;
            setup_backend
                .lock()
                .expect("desktop backend state poisoned")
                .child = child;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Morn desktop shell");

    app.run(move |_handle, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            let _ = exit_backend
                .lock()
                .expect("desktop backend state poisoned")
                .stop();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sibling_backend_lives_beside_desktop_binary() {
        #[cfg(target_os = "windows")]
        let desktop = Path::new(r"C:\morn\morn-desktop.exe");
        #[cfg(not(target_os = "windows"))]
        let desktop = Path::new("/opt/morn/morn-desktop");

        let backend = sibling_backend_path(desktop).unwrap();
        assert_eq!(backend.parent(), desktop.parent());
        #[cfg(target_os = "windows")]
        assert_eq!(backend.file_name().unwrap(), "server.exe");
        #[cfg(not(target_os = "windows"))]
        assert_eq!(backend.file_name().unwrap(), "server");
    }

    #[test]
    fn default_api_base_is_local_reference_backend() {
        assert_eq!(DEFAULT_API_BASE, "http://127.0.0.1:8090");
        assert_eq!(backend_addr(), "127.0.0.1:8090".parse().unwrap());
        assert_eq!(
            DESKTOP_INSTANCE_LOCK_ADDR.parse::<SocketAddr>().unwrap(),
            "127.0.0.1:18091".parse().unwrap()
        );
    }
}
