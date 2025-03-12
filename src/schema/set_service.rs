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
pub struct SetServiceEntry {
    pub key: String,
    pub value: String,
}

impl SetServiceEntry {
    pub fn into_tuple(self) -> (String, String) {
        (self.key, self.value)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SetServiceReq {
    pub id: u64,
    pub service: String,
    pub entries: Vec<SetServiceEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SetServiceRes {
    #[serde(rename = "set")]
    Set,
    #[serde(rename = "error")]
    Error { reason: String },
}

#[cfg(feature = "core")]
impl SetServiceRes {
    pub fn success(_: ()) -> Self {
        Self::Set
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
            SetServiceRes::Set => StatusCode::OK,
            SetServiceRes::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "core")]
impl InternalRouter {
    pub async fn set_service(instance: &ProfileInstance, payload: SetServiceReq) -> SetServiceRes {
        Profile::set_service(
            instance,
            payload.id,
            &payload.service,
            payload
                .entries
                .into_iter()
                .map(SetServiceEntry::into_tuple)
                .collect(),
        )
        .await
        .map(SetServiceRes::success)
        .unwrap_or_else(SetServiceRes::failure)
    }
}

#[cfg(feature = "core")]
impl Router {
    pub async fn set_service(
        State(instance): State<ProfileInstance>,
        Json(payload): Json<SetServiceReq>,
    ) -> (StatusCode, Json<SetServiceRes>) {
        let res = InternalRouter::set_service(&instance, payload).await;
        (res.status(), Json(res))
    }
}
