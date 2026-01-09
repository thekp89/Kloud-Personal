use axum::{extract::State, http::StatusCode};
use std::sync::Arc;
use crate::AppState;

pub async fn get_clipboard(State(state): State<Arc<AppState>>) -> String {
    let clipboard = state.clipboard.read().expect("Lock poisoned");
    clipboard.clone()
}

pub async fn save_clipboard(
    State(state): State<Arc<AppState>>, 
    body: String
) -> StatusCode {
    let mut clipboard = state.clipboard.write().expect("Lock poisoned");
    *clipboard = body;
    StatusCode::OK
}
