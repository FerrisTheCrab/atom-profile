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
pub struct RemoveServiceReq {
    pub id: u64,
    pub service: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RemoveServiceRes {
    #[serde(rename = "removed")]
    Removed,
    #[serde(rename = "error")]
    Error { reason: String },
}

#[cfg(feature = "core")]
impl RemoveServiceRes {
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
            RemoveServiceRes::Removed => StatusCode::OK,
            RemoveServiceRes::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "core")]
impl InternalRouter {
    pub async fn remove_service(
        instance: &ProfileInstance,
        payload: RemoveServiceReq,
    ) -> RemoveServiceRes {
        Profile::remove_service(instance, payload.id, &payload.service)
            .await
            .map(RemoveServiceRes::success)
            .unwrap_or_else(RemoveServiceRes::failure)
    }
}

#[cfg(feature = "core")]
impl Router {
    pub async fn remove_service(
        State(instance): State<ProfileInstance>,
        Json(payload): Json<RemoveServiceReq>,
    ) -> (StatusCode, Json<RemoveServiceRes>) {
        let res = InternalRouter::remove_service(&instance, payload).await;
        (res.status(), Json(res))
    }
}
