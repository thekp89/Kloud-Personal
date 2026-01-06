use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod utils;
mod routes;
mod error;
mod assets;
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

        // Check for dump_theme
        if let Some(path) = &args.dump_theme {
            tracing::info!("Dumping default theme to: {:?}", path);
            if let Err(e) = std::fs::create_dir_all(path) {
                eprintln!("Error creating directory: {}", e);
                std::process::exit(1);
            }

            for file_path in assets::Assets::iter() {
                let embedded_file = assets::Assets::get(&file_path).unwrap();
                let output_path = path.join(file_path.as_ref());
                
                if let Some(parent) = output_path.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                         eprintln!("Error creating subdirectory {:?}: {}", parent, e);
                         continue;
                    }
                }

                if let Err(e) = std::fs::write(&output_path, embedded_file.data) {
                    eprintln!("Error writing file {:?}: {}", output_path, e);
                } else {
                    println!("Extracted: {:?}", output_path);
                }
            }
            println!("Theme dumped successfully.");
            return;
        }

        server::start_server(args).await;
    }
}