use super::GalleryState;
use super::gallery::render;
use super::render::RenderResult;
use crate::server::AppState;
use axum::extract::{Path, Query, State};
use axum::{Router, routing::get};
use serde::Deserialize;

async fn favorite(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
) -> RenderResult {
    let mut state = state;
    state.favorite = state.favorite.or(Some(false)).map(|b| !b);
    render(&app_state, state).await
}

async fn year(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
    Path(year): Path<usize>,
) -> RenderResult {
    let mut state = state;
    if state.year == Some(year) {
        state.year = None;
    } else {
        state.year = Some(year);
    }
    state.month = None;
    render(&app_state, state).await
}

async fn month(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
    Path(month): Path<u8>,
) -> RenderResult {
    let mut state = state;
    assert!(state.year.is_some());
    if state.month == Some(month) {
        state.month = None;
    } else {
        state.month = Some(month);
    }
    render(&app_state, state).await
}

async fn order(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
) -> RenderResult {
    let mut state = state;
    state.new_to_old = state.new_to_old.or(Some(true)).map(|b| !b);
    render(&app_state, state).await
}

async fn collection(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
) -> RenderResult {
    let collection = if state.collection.is_none() {
        let mut root = app_state.config.root_path.clone();
        if !root.ends_with('/') {
            root.push('/');
        }
        Some(root)
    } else {
        None
    };
    let state = GalleryState {
        new_to_old: state.new_to_old,
        collection,
        ..Default::default()
    };
    render(&app_state, state).await
}

#[derive(Deserialize, Debug, Clone)]
struct NavigateTo {
    path: String,
}

async fn navigate(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
    Query(nav): Query<NavigateTo>,
) -> RenderResult {
    let state = GalleryState {
        new_to_old: state.new_to_old,
        collection: Some(nav.path),
        ..Default::default()
    };
    render(&app_state, state).await
}

async fn clear(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
) -> RenderResult {
    let state = GalleryState {
        collection: state.collection,
        ..Default::default()
    };
    render(&app_state, state).await
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/favorite", get(favorite))
        .route("/year/{year}", get(year))
        .route("/month/{month}", get(month))
        .route("/order", get(order))
        .route("/collection", get(collection))
        .route("/navigate", get(navigate))
        .route("/clear", get(clear))
}
