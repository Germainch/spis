use super::GalleryState;
use super::gallery::render;
use super::render::RenderResult;
use crate::server::AppState;
use axum::extract::{Query, State};
use axum::{Router, routing::get};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct NavigateTo {
    path: String,
}

async fn navigate(
    State(app_state): State<AppState>,
    Query(nav): Query<NavigateTo>,
) -> RenderResult {
    let state = GalleryState {
        collection: Some(nav.path),
    };
    render(&app_state, state).await
}

pub fn create_router() -> Router<AppState> {
    Router::new().route("/navigate", get(navigate))
}
