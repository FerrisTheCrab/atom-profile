use axum::routing::post;

use crate::instance::ProfileInstance;

pub struct InternalRouter;
pub struct Router;

impl Router {
    pub fn get(instance: ProfileInstance) -> axum::Router {
        axum::Router::new()
            .route("/remove", post(Router::remove))
            .route("/remove-service", post(Router::remove_service))
            .route("/set", post(Router::set))
            .route("/set-service", post(Router::set_service))
            .route("/show", post(Router::show))
            .route("/show-overlay", post(Router::show_overlay))
            .route("/show-service", post(Router::show_service))
            .with_state(instance)
    }
}
