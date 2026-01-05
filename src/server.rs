use axum::{middleware, routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};


use crate::{auth, routes};

// Definimos la estructura para los argumentos de la línea de comandos
#[derive(Parser, Debug, Clone)] // Added Clone
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// La ruta del directorio que quieres compartir
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// El puerto donde correrá el servidor
    #[arg(short = 'P', long, default_value_t = 3000)]
    pub port: u16,

    /// Tamaño máximo de subida en MB
    #[arg(short = 'S', long, default_value_t = 10)]
    pub max_upload_size: u64,

    /// Habilitar HTTPS (TLS)
    #[arg(long)]
    pub tls: bool,

    /// Ruta al certificado .pem (opcional)
    #[arg(long)]
    pub cert: Option<PathBuf>,

    /// Ruta a la clave privada .key (opcional)
    #[arg(long)]
    pub key: Option<PathBuf>,

    /// Usuario para autenticación básica
    #[arg(short = 'u', long)]
    pub username: Option<String>,

    /// Contraseña para autenticación básica
    #[arg(short = 'w', long)]
    pub password: Option<String>,
}

// Estado compartido
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
        match RustlsConfig::from_pem_file(cert, key).await {
            Ok(config) => return Some(config),
            Err(e) => {
                eprintln!("Error cargando certificados: {}", e);
                std::process::exit(1);
            }
        }
    } else {
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

pub async fn start_server(args: Args) {
    // Validamos que la ruta exista antes de arrancar
    if !args.path.exists() || !args.path.is_dir() {
        eprintln!("Error: La ruta especificada no existe o no es un directorio: {:?}", args.path);
        // In GUI mode, we might want to return an error instead of exiting process, 
        // but for now keeping it simple.
        std::process::exit(1); 
    }

    // Canonicalizamos la ruta
    let base_path = args.path.canonicalize().expect("No se pudo resolver la ruta absoluta");
    
    // Crear el estado compartido
    let state = Arc::new(AppState { 
        base_path: base_path.clone(),
        max_upload_size: args.max_upload_size * 1024 * 1024, // Convertir a bytes
    });

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

    // Configurar TLS
    let tls_config = get_tls_config(&args).await;

    // --- Identity & Discovery ---
    let local_ip = crate::utils::net::get_local_ip();
    let protocol = if tls_config.is_some() { "https" } else { "http" };
    
    // Start mDNS in background
    // We check if it is safe to spawn. In CLI mode, main returns when server finishes,
    // so dropping the handle when this scope ends might be premature if we didn't use `tokio::spawn` inside `register_service`.
    // But `register_service` calls `tokio::spawn` internally and returns a handle to it.
    // If we drop the handle, the task continues unless we explicitly abort it.
    // So we can just let it run.
    let _mdns_service = crate::utils::mdns::register_service(args.port, "local-share", tls_config.is_some());

    // Build Connection URL
    let full_url = crate::utils::net::build_connection_url(
        tls_config.is_some(),
        &local_ip,
        args.port,
        args.username.as_deref(),
        args.password.as_deref(),
        false, // Don't include credentials in CLI text output by default for security, or maybe we want to?
               // The user requested a toggle in GUI. For CLI, maybe just base URL.
               // Let's print the base URL for the text and maybe the autologin one for QR?
               // User said: "Si TLS está activo -> https; si Auth está activa -> incrustar credenciales (opcional por seguridad) o solo la base"
    );

    // For QR, it is convenient to include credentials if present, but risks security.
    // Let's generate the QR with credentials if they exist, but maybe print a warning.
    // Or stick to the plan: "Mitigación: Podrías añadir un "toggle" (interruptor) en la GUI"
    // For CLI, let's use the safer base URL for now unless we want to add a flag. 
    // Wait, the user prompt implies: "El objetivo es convertir la "Cadena de Conexión" en una matriz...".
    // Let's use the base full_url without auth for now to be safe, or check if we want to be fancy.
    // Let's stick to what `build_connection_url` does. I passed `false` above.
    
    tracing::info!("--- Local Share v0.1.0 ---");
    tracing::info!("Local IP Detected: {}", local_ip);
    tracing::info!("Service advertised as: local-share.local");
    tracing::info!("Connection URL: {}", full_url);
    
    // Generate QR
    if let Ok(qr_code) = crate::utils::qr::generate_ascii_qr(&full_url) {
        println!("\nScan this QR code to connect:\n{}", qr_code);
    } else {
        tracing::warn!("Could not generate QR code.");
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    
    if let Some(config) = tls_config {
        tracing::info!("Server listening on {}://0.0.0.0:{}", protocol, args.port);
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        tracing::info!("Server listening on {}://0.0.0.0:{}", protocol, args.port);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}
