use std::path::Path;

use async_trait::async_trait;
#[cfg(feature = "services-core")]
use atom_services::ServiceInstance;
use dyn_clone::DynClone;
use mongodb::{bson::Document, Collection};
#[cfg(feature = "services-request")]
use reqwest::{StatusCode, Url};

use crate::{ConnectionType, MasterConfig, Profile};

#[derive(Clone)]
pub struct ProfileInstance {
    pub config: MasterConfig,
    pub profiles: Collection<Profile>,
    pub profiles_doc: Collection<Document>,
    pub services: Box<dyn ProfileServiceFunctions>,
}

impl ProfileInstance {
    pub fn load(config: &Path) -> Self {
        let config = MasterConfig::read(config);
        let (profiles, profiles_doc) = config.mongodb.load();

        let services: Box<dyn ProfileServiceFunctions> = match &config.services_connection {
            #[cfg(feature = "services-request")]
            ConnectionType::Http { address } => Box::new(ProfileServiceFunctionsRequest::new(
                Url::parse(address).unwrap(),
            )),
            #[cfg(feature = "services-core")]
            ConnectionType::Native { config } => Box::new(ProfileServiceFunctionsCore::new(
                ServiceInstance::load(config),
            )),
        };

        ProfileInstance {
            config,
            profiles,
            profiles_doc,
            services,
        }
    }
}

#[async_trait]
pub trait ProfileServiceFunctions: DynClone + Send + Sync {
    async fn exists(
        &self,
        req: atom_services::schema::ExistsReq,
    ) -> (u16, atom_services::schema::ExistsRes);

    async fn show(
        &self,
        req: atom_services::schema::ShowReq,
    ) -> (u16, atom_services::schema::ShowRes);
}

dyn_clone::clone_trait_object!(ProfileServiceFunctions);

#[cfg(feature = "services-core")]
#[derive(Clone)]
pub struct ProfileServiceFunctionsCore {
    services: atom_services::ServiceInstance,
}

#[cfg(feature = "services-core")]
impl ProfileServiceFunctionsCore {
    pub fn new(services: atom_services::ServiceInstance) -> Self {
        Self { services }
    }
}

#[cfg(feature = "services-core")]
#[async_trait]
impl ProfileServiceFunctions for ProfileServiceFunctionsCore {
    async fn exists(
        &self,
        req: atom_services::schema::ExistsReq,
    ) -> (u16, atom_services::schema::ExistsRes) {
        let res = atom_services::InternalRouter::exists(&self.services, req).await;
        (res.status().as_u16(), res)
    }

    async fn show(
        &self,
        req: atom_services::schema::ShowReq,
    ) -> (u16, atom_services::schema::ShowRes) {
        let res = atom_services::InternalRouter::show(&self.services, req).await;
        (res.status().as_u16(), res)
    }
}

#[cfg(feature = "services-request")]
#[derive(Clone)]
pub struct ProfileServiceFunctionsRequest {
    services: Url,
    reqwest: reqwest::Client,
}

#[cfg(feature = "services-request")]
macro_rules! catch_fail {
    ($t: ident, $x: expr) => {
        match $x {
            Ok(k) => k,
            Err(e) => {
                return (
                    e.status()
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
                        .as_u16(),
                    atom_services::schema::$t::Error {
                        reason: e.to_string(),
                    },
                )
            }
        }
    };
}

#[cfg(feature = "services-request")]
impl ProfileServiceFunctionsRequest {
    pub fn new(services: Url) -> Self {
        Self {
            services,
            reqwest: reqwest::Client::new(),
        }
    }
}

#[cfg(feature = "services-request")]
#[async_trait]
impl ProfileServiceFunctions for ProfileServiceFunctionsRequest {
    async fn exists(
        &self,
        req: atom_services::schema::ExistsReq,
    ) -> (u16, atom_services::schema::ExistsRes) {
        let res = self
            .reqwest
            .post(self.services.join("/api/services/v1/exists").unwrap())
            .json(&req)
            .send()
            .await;

        let res = catch_fail!(ExistsRes, res);
        (
            res.status().as_u16(),
            catch_fail!(ExistsRes, res.json().await),
        )
    }

    async fn show(
        &self,
        req: atom_services::schema::ShowReq,
    ) -> (u16, atom_services::schema::ShowRes) {
        let res = self
            .reqwest
            .post(self.services.join("/api/services/v1/show").unwrap())
            .json(&req)
            .send()
            .await;

        let res = catch_fail!(ShowRes, res);
        (
            res.status().as_u16(),
            catch_fail!(ShowRes, res.json().await),
        )
    }
}
