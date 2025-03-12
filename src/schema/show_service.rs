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
pub struct ShowServiceReq {
    pub id: u64,
    pub service: String,
    pub entries: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ShowServiceRes {
    #[serde(rename = "show")]
    Show { values: BTreeMap<String, String> },
    #[serde(rename = "error")]
    Error { reason: String },
}

#[cfg(feature = "core")]
impl ShowServiceRes {
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
            ShowServiceRes::Show { .. } => StatusCode::OK,
            ShowServiceRes::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "core")]
impl InternalRouter {
    pub async fn show_service(
        instance: &ProfileInstance,
        payload: ShowServiceReq,
    ) -> ShowServiceRes {
        Profile::show_service(instance, payload.id, &payload.service, payload.entries)
            .await
            .map(ShowServiceRes::success)
            .unwrap_or_else(ShowServiceRes::failure)
    }
}

#[cfg(feature = "core")]
impl Router {
    pub async fn show_service(
        State(instance): State<ProfileInstance>,
        Json(payload): Json<ShowServiceReq>,
    ) -> (StatusCode, Json<ShowServiceRes>) {
        let res = InternalRouter::show_service(&instance, payload).await;
        (res.status(), Json(res))
    }
}
