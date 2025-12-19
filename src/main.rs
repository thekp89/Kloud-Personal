use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;
mod routes;
mod utils;
mod auth;
mod server;
mod gui;

pub use server::AppState;

#[tokio::main]
async fn main() {
    // 1. Inicializar el sistema de logs (Tracing)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "local_share=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Decidir modo de ejecución
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() == 1 {
        // Modo GUI (Si no hay argumentos)
        tracing::info!("Iniciando en modo GUI...");
        let options = eframe::NativeOptions::default();
        let _ = eframe::run_native(
            "Local Share Launcher",
            options,
            Box::new(|cc| Ok(Box::new(gui::LocalShareApp::new(cc)))),
        );
    } else {
        // Modo CLI (Si hay argumentos)
        tracing::info!("Iniciando en modo CLI...");
        let args = server::Args::parse();
        server::start_server(args).await;
    }
}