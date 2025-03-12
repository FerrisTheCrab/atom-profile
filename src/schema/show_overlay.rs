use std::collections::BTreeMap;

#[cfg(feature = "core")]
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[cfg(feature = "core")]
use crate::{
    instance::ProfileInstance,
    router::{InternalRouter, Router},
    Profile,
};

#[derive(Serialize, Deserialize)]
pub struct ShowOverlayReq {
    pub id: u64,
    pub service: String,
    pub entries: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ShowOverlayRes {
    #[serde(rename = "show")]
    Show { values: BTreeMap<String, String> },
    #[serde(rename = "error")]
    Error { reason: String },
}

#[cfg(feature = "core")]
impl ShowOverlayRes {
    pub fn success(values: BTreeMap<String, String>) -> Self {
        Self::Show { values }
    }

    pub fn failure(e: mongodb::error::Error) -> Self {
        Self::Error {
            reason: e
                .get_custom::<String>()
                .cloned()
                .unwrap_or(e.kind.to_string()),
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            ShowOverlayRes::Show { .. } => StatusCode::OK,
            ShowOverlayRes::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "core")]
impl InternalRouter {
    pub async fn show_overlay(
        instance: &ProfileInstance,
        payload: ShowOverlayReq,
    ) -> ShowOverlayRes {
        Profile::show_overlay(instance, payload.id, &payload.service, payload.entries)
            .await
            .map(ShowOverlayRes::success)
            .unwrap_or_else(ShowOverlayRes::failure)
    }
}

#[cfg(feature = "core")]
impl Router {
    pub async fn show_overlay(
        State(instance): State<ProfileInstance>,
        Json(payload): Json<ShowOverlayReq>,
    ) -> (StatusCode, Json<ShowOverlayRes>) {
        let res = InternalRouter::show_overlay(&instance, payload).await;
        (res.status(), Json(res))
    }
}
