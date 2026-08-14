//! Morn app server binary.

use std::net::SocketAddr;

use morn_app::{router, AppState};

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
    axum::serve(listener, app).await?;
    Ok(())
}
