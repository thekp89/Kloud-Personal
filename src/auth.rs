use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose, Engine as _};

#[derive(Clone)]
pub struct AuthConfig {
    pub username: String,
    pub password: String,
}

pub async fn auth_middleware(
    config: AuthConfig,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(credentials) = auth_header.strip_prefix("Basic ") {
            if let Ok(decoded) = general_purpose::STANDARD.decode(credentials) {
                if let Ok(decoded_str) = String::from_utf8(decoded) {
                    if let Some((username, password)) = decoded_str.split_once(':') {
                        if username == config.username && password == config.password {
                            return Ok(next.run(req).await);
                        }
                    }
                }
            }
        }
    }

    // Si falla la autenticación, pedimos credenciales
    let response = Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::WWW_AUTHENTICATE, "Basic realm=\"Restricted Area\"")
        .body(Body::empty())
        .unwrap();

    Err(response.status())
}
