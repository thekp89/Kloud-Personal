use axum::{middleware, routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Declaramos los módulos que crearemos más adelante para mantener el orden
mod error;
mod routes;
mod utils;
mod auth;

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

    /// Habilitar HTTPS (TLS)
    #[arg(long)]
    tls: bool,

    /// Ruta al certificado .pem (opcional)
    #[arg(long)]
    cert: Option<PathBuf>,

    /// Ruta a la clave privada .key (opcional)
    #[arg(long)]
    key: Option<PathBuf>,

    /// Usuario para autenticación básica
    #[arg(short = 'u', long)]
    username: Option<String>,

    /// Contraseña para autenticación básica
    #[arg(short = 'w', long)]
    password: Option<String>,
}

// Estado compartido que pasaremos a todos los handlers (endpoints)
// Usamos Arc (Atomic Reference Counting) para que sea seguro leerlo desde múltiples hilos
#[derive(Clone)]
pub struct AppState {
    pub base_path: PathBuf,
    pub max_upload_size: u64,
}

async fn get_tls_config(args: &Args) -> Option<RustlsConfig> {
    if !args.tls {
        return None;
    }

    if let (Some(cert), Some(key)) = (&args.cert, &args.key) {
        // Usar certificados provistos por el usuario
        match RustlsConfig::from_pem_file(cert, key).await {
            Ok(config) => return Some(config),
            Err(e) => {
                eprintln!("Error cargando certificados: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Generar certificado efímero en memoria
        tracing::warn!("Generando certificado autofirmado efímero...");
        let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string(), "0.0.0.0".to_string()];
        let cert = rcgen::generate_simple_self_signed(subject_alt_names).unwrap();
        let cert_pem = cert.serialize_pem().unwrap();
        let key_pem = cert.serialize_private_key_pem();

        let config = RustlsConfig::from_pem(
            cert_pem.into_bytes(),
            key_pem.into_bytes(),
        ).await.unwrap();
        
        return Some(config);
    }
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
    
    // 3. Crear el estado compartido
    let state = Arc::new(AppState { 
        base_path: base_path.clone(),
        max_upload_size: args.max_upload_size * 1024 * 1024, // Convertir a bytes
    });

    // 4. Configurar el Router
    let mut app = Router::new()
        .route("/health", get(|| async { "Servidor activo" }))
        .merge(routes::app_router())
        .layer(axum::extract::DefaultBodyLimit::max((args.max_upload_size * 1024 * 1024) as usize))
        .with_state(state);

    // Middleware de Autenticación Condicional
    if let (Some(username), Some(password)) = (args.username.clone(), args.password.clone()) {
        tracing::info!("Autenticación habilitada para usuario: {}", username);
        let auth_config = auth::AuthConfig { username, password };
        app = app.layer(middleware::from_fn(move |req, next| {
            auth::auth_middleware(auth_config.clone(), req, next)
        }));
    }

    // 5. Configurar TLS
    let tls_config = get_tls_config(&args).await;

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    
    if let Some(config) = tls_config {
        tracing::info!("Compartiendo carpeta: {:?} (HTTPS)", base_path);
        tracing::info!("Servidor escuchando en https://0.0.0.0:{}", args.port);
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        tracing::info!("Compartiendo carpeta: {:?} (HTTP)", base_path);
        tracing::info!("Servidor escuchando en http://0.0.0.0:{}", args.port);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}