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
pub struct ShowReq {
    pub id: u64,
    pub entries: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ShowRes {
    #[serde(rename = "show")]
    Show { values: BTreeMap<String, String> },
    #[serde(rename = "error")]
    Error { reason: String },
}

#[cfg(feature = "core")]
impl ShowRes {
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
            ShowRes::Show { .. } => StatusCode::OK,
            ShowRes::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "core")]
impl InternalRouter {
    pub async fn show(instance: &ProfileInstance, payload: ShowReq) -> ShowRes {
        Profile::show(instance, payload.id, payload.entries)
            .await
            .map(ShowRes::success)
            .unwrap_or_else(ShowRes::failure)
    }
}

#[cfg(feature = "core")]
impl Router {
    pub async fn show(
        State(instance): State<ProfileInstance>,
        Json(payload): Json<ShowReq>,
    ) -> (StatusCode, Json<ShowRes>) {
        let res = InternalRouter::show(&instance, payload).await;
        (res.status(), Json(res))
    }
}
