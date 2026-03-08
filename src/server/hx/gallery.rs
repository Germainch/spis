use crate::db::{self, Order};
use crate::server::AppState;

use super::Cursor;
use super::GalleryState;
use super::Media;
use super::Subdir;
use super::render::RenderResult;
use super::render::ServerError;
use super::render::TemplatedResponse;
use askama::Template;
use axum::extract::{Query, State};
use axum::{Router, routing::get};

const PAGE_SIZE: usize = 400;

mod filters {
    #![allow(clippy::inline_always, clippy::unused_self)]

    use core::fmt;

    use super::super::{Cursor, Media, gallery::PAGE_SIZE};

    const EMPTY: String = String::new();

    #[askama::filter_fn]
    pub fn cursor(media: &[Media], _: &dyn askama::Values) -> askama::Result<String> {
        if media.len() < PAGE_SIZE {
            Ok(EMPTY)
        } else {
            Ok(serde_urlencoded::to_string(Cursor {
                cursor: media.last().expect("List should not be empty").taken_at,
            })
            .map_err(|_| fmt::Error)?)
        }
    }
}

#[derive(Template)]
#[template(path = "web/gallery/gallery.html")]
struct HxGallery<'a> {
    collection: &'a str,
    features: &'a crate::server::Features,
    subdirs: &'a Vec<Subdir>,
    breadcrumbs: &'a Vec<(String, String)>,

    media: &'a Vec<Media>,
}

pub(super) async fn render(app_state: &AppState, state: GalleryState) -> RenderResult {
    let pool = &app_state.pool;
    let config = &app_state.config;

    let root = {
        let mut r = config.root_path.clone();
        if !r.ends_with('/') {
            r.push('/');
        }
        r
    };

    let collection = state.collection.clone().unwrap_or_else(|| root.clone());
    let state = GalleryState {
        collection: Some(collection.clone()),
    };

    let subdirs: Vec<Subdir> = db::subdirs_list(pool, &collection)
        .await
        .map_err(ServerError::DB)?
        .into_iter()
        .map(|name| {
            let path = format!("{collection}{name}");
            let display_name = name.trim_end_matches('/').to_string();
            Subdir {
                name: display_name,
                path,
            }
        })
        .collect();

    let relative = collection
        .strip_prefix(root.as_str())
        .unwrap_or(&collection);
    let mut breadcrumbs = vec![("/".to_string(), root.clone())];
    let mut accumulated = root.clone();
    for segment in relative.split('/').filter(|s| !s.is_empty()) {
        accumulated.push_str(segment);
        accumulated.push('/');
        breadcrumbs.push((segment.to_string(), accumulated.clone()));
    }

    let media = db::media_list(pool, &state, Order::Desc, PAGE_SIZE)
        .await
        .map_err(ServerError::DB)?
        .into_iter()
        .map(|row| (row, &config.pathfinder).into())
        .collect();

    HxGallery {
        collection: &collection,
        features: &config.features,
        subdirs: &subdirs,
        breadcrumbs: &breadcrumbs,
        media: &media,
    }
    .render_response()
}

async fn root(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
) -> RenderResult {
    render(&app_state, state).await
}

#[derive(Template)]
#[template(path = "web/gallery/list.html")]
struct HxMore<'a> {
    features: &'a crate::server::Features,
    media: &'a Vec<Media>,
}

async fn more(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
    Query(cursor): Query<Cursor>,
) -> RenderResult {
    let pool = &app_state.pool;
    let config = &app_state.config;
    let media = db::media_list(pool, (&state, &cursor), Order::Desc, PAGE_SIZE)
        .await
        .map_err(ServerError::DB)?
        .into_iter()
        .map(|row| (row, &config.pathfinder).into())
        .collect();

    HxMore {
        features: &config.features,
        media: &media,
    }
    .render_response()
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", get(root))
        .route("/more", get(more))
}
