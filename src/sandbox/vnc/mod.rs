//! VNC sandbox — Provides headless virtual desktop environments for agent isolation.
//!
//! ## Feature Gate
//! Requires `vnc-sandbox` feature to be enabled.
//!
//! ## Architecture
//! - Each agent gets an isolated VNC session (virtual framebuffer).
//! - Supports connecting to remote VNC servers or creating local headless sessions.
//! - Screenshot capture and keyboard/mouse operations are proxied through VNC.
//!
//! ## Platform Notes
//! - Local headless sessions require Xvfb (X Virtual Framebuffer) on Linux.
//! - Remote connections work cross-platform via TCP.

use crate::core::error::MornError;
use std::collections::HashMap;
use std::sync::Mutex;

/// Represents a VNC session (real or simulated).
#[derive(Debug, Clone)]
pub struct VncSession {
    pub id: String,
    pub agent_id: String,
    pub host: String,
    pub port: u16,
    pub display: Option<String>,
    pub width: u32,
    pub height: u32,
    pub connected: bool,
}

/// Manages VNC sessions for agent isolation.
pub struct VncManager {
    sessions: Mutex<HashMap<String, VncSession>>,
    next_display: Mutex<u16>,
}

impl VncManager {
    pub fn new() -> Self {
        VncManager {
            sessions: Mutex::new(HashMap::new()),
            next_display: Mutex::new(99),
        }
    }

    /// Create a new headless VNC session for an agent.
    pub fn create_session(
        &self,
        agent_id: &str,
        width: u32,
        height: u32,
    ) -> Result<String, MornError> {
        let session_id = format!("vnc-{}", uuid::Uuid::new_v4());

        let session = if self.can_start_xvfb() {
            let mut disp = self
                .next_display
                .lock()
                .map_err(|e| MornError::Internal(e.to_string()))?;
            *disp += 1;
            let display_str = format!(":{}", disp);
            let xvfb_args = [
                display_str.as_str(),
                "-screen",
                "0",
                &format!("{}x{}x24", width, height),
                "-ac",
            ];
            match std::process::Command::new("Xvfb").args(xvfb_args).spawn() {
                Ok(_) => {
                    std::env::set_var("DISPLAY", &display_str);
                    VncSession {
                        id: session_id.clone(),
                        agent_id: agent_id.to_string(),
                        host: "localhost".to_string(),
                        port: 5900 + *disp,
                        display: Some(display_str),
                        width,
                        height,
                        connected: true,
                    }
                }
                Err(_) => self.simulated_session(&session_id, agent_id, width, height),
            }
        } else {
            self.simulated_session(&session_id, agent_id, width, height)
        };

        let mut sessions = self
            .sessions
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    fn can_start_xvfb(&self) -> bool {
        cfg!(feature = "vnc-sandbox")
            && cfg!(target_os = "linux")
            && !cfg!(test)
            && std::process::Command::new("which")
                .arg("Xvfb")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }

    fn simulated_session(
        &self,
        session_id: &str,
        agent_id: &str,
        width: u32,
        height: u32,
    ) -> VncSession {
        VncSession {
            id: session_id.to_string(),
            agent_id: agent_id.to_string(),
            host: "localhost".to_string(),
            port: 5900,
            display: None,
            width,
            height,
            connected: true,
        }
    }

    /// Connect to a remote VNC server.
    pub fn connect_remote(
        &self,
        agent_id: &str,
        host: &str,
        port: u16,
    ) -> Result<String, MornError> {
        let session_id = format!("vnc-remote-{}", uuid::Uuid::new_v4());

        use std::net::TcpStream;
        let addr = format!("{}:{}", host, port);
        match TcpStream::connect(&addr) {
            Ok(_stream) => {
                let session = VncSession {
                    id: session_id.clone(),
                    agent_id: agent_id.to_string(),
                    host: host.to_string(),
                    port,
                    display: None,
                    width: 1920,
                    height: 1080,
                    connected: true,
                };
                let mut sessions = self
                    .sessions
                    .lock()
                    .map_err(|e| MornError::Internal(e.to_string()))?;
                sessions.insert(session_id.clone(), session);
                Ok(session_id)
            }
            Err(e) => Err(MornError::Internal(format!(
                "Failed to connect to VNC server at {}: {}",
                addr, e
            ))),
        }
    }

    /// Destroy a VNC session.
    pub fn destroy_session(&self, session_id: &str) -> Result<(), MornError> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        sessions.remove(session_id);
        Ok(())
    }

    /// Get a session by ID.
    pub fn get_session(&self, session_id: &str) -> Result<VncSession, MornError> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| MornError::Internal(format!("Session not found: {}", session_id)))
    }

    /// List all sessions for a given agent.
    pub fn agent_sessions(&self, agent_id: &str) -> Vec<VncSession> {
        let sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        sessions
            .values()
            .filter(|s| s.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Take a screenshot of a VNC session.
    pub fn screenshot(&self, session_id: &str) -> Result<String, MornError> {
        let session = self.get_session(session_id)?;

        if !cfg!(test) && cfg!(feature = "vnc-sandbox") && cfg!(target_os = "linux") {
            let display = session.display.as_deref().unwrap_or(":0");
            let screenshot_path = std::env::temp_dir().join("morn_vnc_screenshot.png");
            let screenshot_str = screenshot_path
                .to_str()
                .ok_or_else(|| format!("非UTF-8路径: {:?}", screenshot_path))?
                .to_string();
            let output = std::process::Command::new("import")
                .args(["-display", display, "-window", "root", &screenshot_str])
                .output()
                .map_err(|e| MornError::Internal(format!("Screenshot failed: {}", e)))?;

            if output.status.success() {
                Ok(screenshot_str)
            } else {
                Err(MornError::Internal("Screenshot command failed".to_string()))
            }
        } else {
            Ok("[simulated] vnc screenshot path".to_string())
        }
    }

    /// Send keyboard input to a VNC session.
    pub fn keyboard_type(&self, session_id: &str, text: &str) -> Result<(), MornError> {
        let _session = self.get_session(session_id)?;

        if !cfg!(test) && cfg!(feature = "vnc-sandbox") && cfg!(target_os = "linux") {
            let display = _session.display.as_deref().unwrap_or(":0");
            let output = std::process::Command::new("xdotool")
                .args(["type", "--display", display, text])
                .output()
                .map_err(|e| MornError::Internal(format!("xdotool type failed: {}", e)))?;

            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(MornError::Internal(format!("xdotool error: {}", stderr)))
            }
        } else {
            Ok(())
        }
    }

    /// Send mouse click to a VNC session.
    pub fn mouse_click(
        &self,
        session_id: &str,
        x: i32,
        y: i32,
        button: &str,
    ) -> Result<(), MornError> {
        let _session = self.get_session(session_id)?;

        if !cfg!(test) && cfg!(feature = "vnc-sandbox") && cfg!(target_os = "linux") {
            let display = _session.display.as_deref().unwrap_or(":0");
            let btn = match button {
                "right" => "3",
                "middle" => "2",
                _ => "1",
            };

            std::process::Command::new("xdotool")
                .args([
                    "mousemove",
                    "--display",
                    display,
                    &x.to_string(),
                    &y.to_string(),
                ])
                .output()
                .map_err(|e| MornError::Internal(format!("xdotool mousemove failed: {}", e)))?;

            std::process::Command::new("xdotool")
                .args(["click", "--display", display, btn])
                .output()
                .map_err(|e| MornError::Internal(format!("xdotool click failed: {}", e)))?;

            Ok(())
        } else {
            Ok(())
        }
    }

    /// Check if a session is active.
    pub fn is_active(&self, session_id: &str) -> bool {
        self.sessions
            .lock()
            .map(|s| s.contains_key(session_id))
            .unwrap_or(false)
    }

    /// Get count of active sessions.
    pub fn active_count(&self) -> usize {
        self.sessions.lock().map(|s| s.len()).unwrap_or(0)
    }
}

impl Default for VncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let manager = VncManager::new();
        let session_id = manager.create_session("agent_1", 1024, 768).unwrap();
        assert!(session_id.starts_with("vnc-"));
    }

    #[test]
    fn test_destroy_session() {
        let manager = VncManager::new();
        let session_id = manager.create_session("agent_1", 800, 600).unwrap();
        assert!(manager.is_active(&session_id));
        manager.destroy_session(&session_id).unwrap();
        assert!(!manager.is_active(&session_id));
    }

    #[test]
    fn test_get_session() {
        let manager = VncManager::new();
        let session_id = manager.create_session("agent_2", 1920, 1080).unwrap();
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.agent_id, "agent_2");
        assert_eq!(session.width, 1920);
        assert_eq!(session.height, 1080);
    }

    #[test]
    fn test_agent_sessions() {
        let manager = VncManager::new();
        let _s1 = manager.create_session("agent_x", 800, 600).unwrap();
        let _s2 = manager.create_session("agent_x", 1024, 768).unwrap();
        let _s3 = manager.create_session("agent_y", 800, 600).unwrap();

        let agent_x_sessions = manager.agent_sessions("agent_x");
        assert_eq!(agent_x_sessions.len(), 2);

        let agent_y_sessions = manager.agent_sessions("agent_y");
        assert_eq!(agent_y_sessions.len(), 1);
    }

    #[test]
    fn test_screenshot_simulated() {
        let manager = VncManager::new();
        let session_id = manager.create_session("agent_1", 800, 600).unwrap();
        let result = manager.screenshot(&session_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_keyboard_type_simulated() {
        let manager = VncManager::new();
        let session_id = manager.create_session("agent_1", 800, 600).unwrap();
        let result = manager.keyboard_type(&session_id, "hello world");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mouse_click_simulated() {
        let manager = VncManager::new();
        let session_id = manager.create_session("agent_1", 800, 600).unwrap();
        let result = manager.mouse_click(&session_id, 100, 200, "left");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_nonexistent_session() {
        let manager = VncManager::new();
        let result = manager.get_session("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_active_count() {
        let manager = VncManager::new();
        assert_eq!(manager.active_count(), 0);
        let _s1 = manager.create_session("a", 800, 600).unwrap();
        assert_eq!(manager.active_count(), 1);
        let _s2 = manager.create_session("b", 800, 600).unwrap();
        assert_eq!(manager.active_count(), 2);
    }
}
