use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use mongodb::{
    bson::Document,
    options::{AuthMechanism, ClientOptions, Credential},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

use crate::Profile;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ConnectionType {
    #[cfg(feature = "services-request")]
    #[serde(rename = "http")]
    Http { address: String },
    #[cfg(feature = "services-core")]
    #[serde(rename = "native")]
    Native { config: std::path::PathBuf },
}

impl ConnectionType {
    pub fn services() -> Self {
        #[cfg(feature = "services-request")]
        return Self::Http {
            address: "localhost:8080".to_string(),
        };
        #[cfg(all(feature = "services-core", not(feature = "services-request")))]
        return ConnectionType::Native {
            config: "services.json".into(),
        };
    }
}

#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde, Clone)]
pub struct MasterConfig {
    #[serde_inline_default(8080)]
    pub port: u16,
    #[serde_inline_default(ConnectionType::services())]
    #[serde(rename = "services-connection")]
    pub services_connection: ConnectionType,
    #[serde(default)]
    pub mongodb: MongoConfig,
}

#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde, Clone)]
pub struct MongoConfig {
    #[serde_inline_default("mongodb://localhost:27017".to_string())]
    pub address: String,
    #[serde_inline_default("bob".to_string())]
    pub username: String,
    #[serde_inline_default("cratchit".to_string())]
    pub password: String,
    #[serde_inline_default("admin".to_string())]
    #[serde(rename = "authDB")]
    pub auth_db: String,
    #[serde_inline_default("atomics".to_string())]
    #[serde(rename = "masterDB")]
    pub master_db: String,
}

impl MasterConfig {
    fn create(path: &Path) {
        let ser = serde_json::to_vec_pretty(&Self::default()).unwrap();

        if !path.parent().unwrap().exists() {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
        }

        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap()
            .write_all(&ser)
            .unwrap();
    }

    pub fn read(path: &Path) -> Self {
        if !path.exists() {
            Self::create(path);
        }

        let content = fs::read(path).unwrap();
        serde_json::from_slice(&content).expect("bad JSON")
    }
}

impl MongoConfig {
    pub fn load(&self) -> (Collection<Profile>, Collection<Document>) {
        futures::executor::block_on(async {
            (
                self.get_collection().await,
                self.get_collection_docs().await,
            )
        })
    }

    async fn get_collection(&self) -> Collection<Profile> {
        let mut client_opts = ClientOptions::parse(&self.address).await.unwrap();

        let scram_sha_1_cred = Credential::builder()
            .username(self.username.clone())
            .password(self.password.clone())
            .mechanism(AuthMechanism::ScramSha1)
            .source(self.auth_db.clone())
            .build();

        client_opts.credential = Some(scram_sha_1_cred);
        let client = Client::with_options(client_opts).unwrap();
        client.database(&self.master_db).collection("profile")
    }

    async fn get_collection_docs(&self) -> Collection<Document> {
        let mut client_opts = ClientOptions::parse(&self.address).await.unwrap();

        let scram_sha_1_cred = Credential::builder()
            .username(self.username.clone())
            .password(self.password.clone())
            .mechanism(AuthMechanism::ScramSha1)
            .source(self.auth_db.clone())
            .build();

        client_opts.credential = Some(scram_sha_1_cred);
        let client = Client::with_options(client_opts).unwrap();
        client.database(&self.master_db).collection("profile")
    }
}
