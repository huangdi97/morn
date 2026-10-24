//! Morn app server binary.

use std::net::SocketAddr;

use morn_app::{router, AppState};
use tokio::io::AsyncReadExt;

async fn shutdown_signal() {
    if std::env::var("MORN_EXIT_ON_STDIN_CLOSE").as_deref() == Ok("1") {
        // Desktop-owned sidecars inherit a private stdin pipe. EOF means the
        // owning desktop exited (normally or abnormally), so the local server
        // can drain Axum gracefully instead of becoming an orphan process.
        let mut stdin = tokio::io::stdin();
        let mut byte = [0_u8; 1];
        loop {
            match stdin.read(&mut byte).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
        }
        return;
    }

    // Standalone server behavior remains conventional and independent of the
    // desktop lifecycle.
    let _ = tokio::signal::ctrl_c().await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = std::env::var("MORN_DB").unwrap_or_else(|_| "morn.db".to_string());
    let port: u16 = std::env::var("MORN_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8090);

    let state = AppState::new(&db_path)?;
    let app = router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Morn API listening on http://{addr} (db={db_path})");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}
