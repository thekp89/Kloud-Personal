use axum::{routing::get, Router};
use clap::Parser;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Declaramos los módulos que crearemos más adelante para mantener el orden
mod error;
mod routes;
mod utils;

// Definimos la estructura para los argumentos de la línea de comandos
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// La ruta del directorio que quieres compartir
    #[arg(short, long, default_value = ".")]
    path: PathBuf,

    /// El puerto donde correrá el servidor
    #[arg(short = 'P', long, default_value_t = 3000)]
    port: u16,

    /// Tamaño máximo de subida en MB
    #[arg(short = 'S', long, default_value_t = 10)]
    max_upload_size: u64,
}

// Estado compartido que pasaremos a todos los handlers (endpoints)
// Usamos Arc (Atomic Reference Counting) para que sea seguro leerlo desde múltiples hilos
#[derive(Clone)]
pub struct AppState {
    pub base_path: PathBuf,
    pub max_upload_size: u64,
}

#[tokio::main]
async fn main() {
    // 1. Inicializar el sistema de logs (Tracing)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "local_share=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Parsear argumentos
    let args = Args::parse();

    // Validamos que la ruta exista antes de arrancar
    if !args.path.exists() || !args.path.is_dir() {
        eprintln!("Error: La ruta especificada no existe o no es un directorio: {:?}", args.path);
        std::process::exit(1);
    }

    // Canonicalizamos la ruta (resolvemos rutas relativas como ".." o ".") a una ruta absoluta
    let base_path = args.path.canonicalize().expect("No se pudo resolver la ruta absoluta");
    
    tracing::info!("Compartiendo carpeta: {:?}", base_path);
    tracing::info!("Servidor escuchando en http://0.0.0.0:{}", args.port);

    // 3. Crear el estado compartido
    let state = Arc::new(AppState { 
        base_path,
        max_upload_size: args.max_upload_size * 1024 * 1024, // Convertir a bytes
    });

    // 4. Configurar el Router
    // Aquí es donde conectaremos nuestros módulos de rutas más adelante.
    // Por ahora, ponemos un "health check" simple.
    let app = Router::new()
        .route("/health", get(|| async { "Servidor activo" }))
        .merge(routes::app_router()) // Descomentaremos esto cuando creemos el módulo routes
        .layer(axum::extract::DefaultBodyLimit::max((args.max_upload_size * 1024 * 1024) as usize))
        .with_state(state);

    // 5. Levantar el servidor
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Error en el servidor: {}", e);
    }
}