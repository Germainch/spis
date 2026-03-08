use askama::Template;
use axum::{Router, routing::get};
use chrono::{DateTime, Utc};
use render::{RenderResult, TemplatedResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::server::AppState;
use crate::{PathFinder, db::Filter, db::MediaRow};

use std::path::Path;

mod actions;
mod bar;
mod gallery;
mod preview;
mod render;

#[derive(Clone)]
pub struct Media {
    pub uuid: Uuid,
    pub url: String,
    pub name: String,
    pub thumbnail: String,
    pub path: String,
    pub favorite: bool,
    pub video: bool,
    pub taken_at: DateTime<Utc>,
}

impl From<(MediaRow, &PathFinder)> for Media {
    fn from(value: (MediaRow, &PathFinder)) -> Self {
        let name = Path::new(&value.0.path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&value.0.path)
            .to_string();
        Self {
            uuid: value.0.id,
            url: value.1.media(&value.0.path),
            name,
            thumbnail: value.1.thumbnail(&value.0.id),
            path: value.0.path,
            favorite: value.0.favorite,
            video: value.0.media_type == 1,
            taken_at: value.0.taken_at,
        }
    }
}

pub struct Subdir {
    pub name: String,
    pub path: String,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub(super) struct GalleryState {
    collection: Option<String>,
}

impl From<&GalleryState> for Filter {
    fn from(value: &GalleryState) -> Self {
        Self {
            archived: false,
            favorite: None,
            taken_after: None,
            taken_before: None,
            collection: value.collection.clone(),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Cursor {
    cursor: DateTime<Utc>,
}

impl From<(&GalleryState, &Cursor)> for Filter {
    fn from(value: (&GalleryState, &Cursor)) -> Self {
        let mut filter: Self = value.0.into();
        filter.taken_before = Some(value.1.cursor);
        filter
    }
}

const fn dev_enabled() -> bool {
    #[cfg(feature = "dev")]
    let dev = true;
    #[cfg(not(feature = "dev"))]
    let dev = false;
    dev
}

#[derive(Template)]
#[template(path = "web/index.html")]
struct HxIndex {}

async fn index() -> RenderResult {
    HxIndex {}.render_response()
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .nest("/action", actions::create_router())
        .nest("/bar", bar::create_router())
        .nest("/gallery", gallery::create_router())
        .nest("/preview", preview::create_router())
}
