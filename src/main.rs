use std::error::Error;
use thruster::{HyperServer, ThrusterServer};
use tracing::info;

mod app;
mod controllers;
mod errors;
mod models;
mod services;
mod thruster_extensions;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("Could not parse PORT");

    let server = HyperServer::new(app::init().await);
    info!("Starting on port {port}");

    server.build("0.0.0.0", port).await;

    Ok(())
}
