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
pub struct SetEntry {
    pub key: String,
    pub value: String,
}

impl SetEntry {
    pub fn into_tuple(self) -> (String, String) {
        (self.key, self.value)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SetReq {
    pub id: u64,
    pub entries: Vec<SetEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SetRes {
    #[serde(rename = "set")]
    Set,
    #[serde(rename = "error")]
    Error { reason: String },
}

#[cfg(feature = "core")]
impl SetRes {
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
            SetRes::Set => StatusCode::OK,
            SetRes::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "core")]
impl InternalRouter {
    pub async fn set(instance: &ProfileInstance, payload: SetReq) -> SetRes {
        Profile::set(
            instance,
            payload.id,
            payload
                .entries
                .into_iter()
                .map(SetEntry::into_tuple)
                .collect(),
        )
        .await
        .map(SetRes::success)
        .unwrap_or_else(SetRes::failure)
    }
}

#[cfg(feature = "core")]
impl Router {
    pub async fn set(
        State(instance): State<ProfileInstance>,
        Json(payload): Json<SetReq>,
    ) -> (StatusCode, Json<SetRes>) {
        let res = InternalRouter::set(&instance, payload).await;
        (res.status(), Json(res))
    }
}
