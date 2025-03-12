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
pub struct RemoveReq {
    pub id: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RemoveRes {
    #[serde(rename = "removed")]
    Removed,
    #[serde(rename = "error")]
    Error { reason: String },
}

#[cfg(feature = "core")]
impl RemoveRes {
    pub fn success(_: ()) -> Self {
        Self::Removed
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
            RemoveRes::Removed => StatusCode::OK,
            RemoveRes::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "core")]
impl InternalRouter {
    pub async fn remove(instance: &ProfileInstance, payload: RemoveReq) -> RemoveRes {
        Profile::remove(instance, payload.id)
            .await
            .map(RemoveRes::success)
            .unwrap_or_else(RemoveRes::failure)
    }
}

#[cfg(feature = "core")]
impl Router {
    pub async fn remove(
        State(instance): State<ProfileInstance>,
        Json(payload): Json<RemoveReq>,
    ) -> (StatusCode, Json<RemoveRes>) {
        let res = InternalRouter::remove(&instance, payload).await;
        (res.status(), Json(res))
    }
}
