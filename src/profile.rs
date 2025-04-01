use std::collections::BTreeMap;

use atom_services::schema::{ExistsReq, ExistsRes};
use mongodb::bson::{doc, Bson, Document};
use serde::{Deserialize, Serialize};

use crate::instance::ProfileInstance;

macro_rules! opt_unwrap {
    ($x: expr) => {
        match $x {
            Some(pr) => pr,
            None => return not_found!(),
        }
    };
}

macro_rules! not_found {
    () => {
        Err(mongodb::error::Error::custom(
            "profile not found".to_string(),
        ))
    };
}

macro_rules! no_service {
    () => {
        Err(mongodb::error::Error::custom(
            "service not found".to_string(),
        ))
    };
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Profile {
    #[serde(rename = "_id")]
    id: u64,
    bucket: BTreeMap<String, String>,
    services: BTreeMap<String, BTreeMap<String, String>>,
}

impl Profile {
    fn encode(s: &str) -> String {
        let mut out = String::new();

        for c in s.chars() {
            match c {
                '$' => out.push_str("$d"),
                '.' => out.push_str("$p"),
                c => out.push(c),
            }
        }

        out
    }

    fn decode(s: &str) -> String {
        let mut out = String::new();
        let mut op = false;

        for c in s.chars() {
            match c {
                '$' => op = true,
                c if op => {
                    match c {
                        'd' => out.push('$'),
                        'p' => out.push('.'),
                        _ => panic!("unknown escape sequence ${c}"),
                    };
                    op = false
                }
                c => out.push(c),
            }
        }

        out
    }
}

impl Profile {
    async fn set_int(
        instance: &ProfileInstance,
        id: u64,
        entries: Vec<(String, String)>,
    ) -> Result<(), mongodb::error::Error> {
        let mut m_set = Document::new();
        let mut m_unset = Document::new();

        for (k, v) in entries.iter() {
            if v.is_empty() {
                m_unset.insert(format!("bucket.{}", Self::encode(k)), v.to_string());
            } else {
                m_set.insert(format!("bucket.{}", Self::encode(k)), v.to_string());
            }
        }

        if instance
            .profiles
            .update_one(
                doc! { "_id": Bson::Int64(id as i64) },
                doc! { "$set": m_set , "$unset": m_unset},
            )
            .await?
            .matched_count
            == 0
        {
            instance
                .profiles
                .insert_one(Profile {
                    id,
                    bucket: entries.into_iter().collect(),
                    services: BTreeMap::new(),
                })
                .await?;
        }

        Ok(())
    }

    async fn get_int(
        instance: &ProfileInstance,
        id: u64,
        entries: Vec<String>,
    ) -> Result<BTreeMap<String, String>, mongodb::error::Error> {
        let mut projection = Document::new();

        for k in entries.into_iter() {
            projection.insert(Self::encode(&k), 1);
        }

        Ok(opt_unwrap!(
            instance
                .profiles_doc
                .find_one(doc! { "_id": Bson::Int64(id as i64)})
                .projection(doc! { "bucket": projection })
                .await?
        )
        .get_document("bucket")
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|(k, v)| (Self::decode(&k), v.as_str().unwrap().to_string()))
        .collect())
    }

    async fn remove_int(instance: &ProfileInstance, id: u64) -> Result<(), mongodb::error::Error> {
        if instance
            .profiles
            .delete_one(doc! { "_id": Bson::Int64(id as i64) })
            .await?
            .deleted_count
            == 0
        {
            return not_found!();
        }

        Ok(())
    }

    async fn set_service_int(
        instance: &ProfileInstance,
        id: u64,
        service: &str,
        entries: Vec<(String, String)>,
    ) -> Result<(), mongodb::error::Error> {
        let (_, res) = instance
            .services
            .exists(ExistsReq {
                id: service.to_string(),
            })
            .await;

        match res {
            ExistsRes::Exists { value: false } => return no_service!(),
            ExistsRes::Exists { value: true } => {}
            ExistsRes::Error { reason } => return Err(mongodb::error::Error::custom(reason)),
        }

        let mut m_set = Document::new();
        let mut m_unset = Document::new();

        for (k, v) in entries.iter() {
            if v.is_empty() {
                m_unset.insert(format!("services.{service}.{}", Self::encode(k)), v.to_string());
            } else {
                m_set.insert(format!("services.{service}.{}", Self::encode(k)), v.to_string());
            }
        }

        if instance
            .profiles
            .update_one(
                doc! { "_id": Bson::Int64(id as i64) },
                doc! { "$set": m_set, "$unset": m_unset},
            )
            .await?
            .matched_count
            == 1
        {
            Ok(())
        } else {
            not_found!()
        }
    }

    async fn get_service_int(
        instance: &ProfileInstance,
        id: u64,
        service: &str,
        entries: Vec<String>,
    ) -> Result<BTreeMap<String, String>, mongodb::error::Error> {
        let (_, res) = instance
            .services
            .exists(ExistsReq {
                id: service.to_string(),
            })
            .await;

        match res {
            ExistsRes::Exists { value: false } => return no_service!(),
            ExistsRes::Exists { value: true } => {}
            ExistsRes::Error { reason } => return Err(mongodb::error::Error::custom(reason)),
        }

        let mut projection = Document::new();

        for k in entries.into_iter() {
            projection.insert(Self::encode(&k), 1);
        }

        Ok(opt_unwrap!(
            instance
                .profiles_doc
                .find_one(doc! { "_id": Bson::Int64(id as i64)})
                .projection(doc! { "services": doc!{ service: projection}})
                .await?
        )
        .get_document("services")
        .cloned()
        .unwrap_or_default()
        .get_document(service)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|(k, v)| (Self::decode(&k), v.as_str().unwrap().to_string()))
        .collect())
    }

    async fn remove_service_int(
        instance: &ProfileInstance,
        id: u64,
        service: &str,
    ) -> Result<(), mongodb::error::Error> {
        let (_, res) = instance
            .services
            .exists(ExistsReq {
                id: service.to_string(),
            })
            .await;

        match res {
            ExistsRes::Exists { value: false } => return no_service!(),
            ExistsRes::Exists { value: true } => {}
            ExistsRes::Error { reason } => return Err(mongodb::error::Error::custom(reason)),
        }

        if instance
            .profiles
            .update_one(
                doc! { "_id": Bson::Int64(id as i64) },
                doc! { "$unset": doc!{ "services": service}},
            )
            .await?
            .matched_count
            == 0
        {
            not_found!()
        } else {
            Ok(())
        }
    }

    async fn get_overlay_int(
        instance: &ProfileInstance,
        id: u64,
        service: &str,
        entries: Vec<String>,
    ) -> Result<BTreeMap<String, String>, mongodb::error::Error> {
        let mut service_entries =
            Self::get_service_int(instance, id, service, entries.clone()).await?;

        let remaining = entries
            .into_iter()
            .filter(|s| !service_entries.contains_key(s))
            .collect::<Vec<_>>();
        let global_entries = Self::get_int(instance, id, remaining).await?;
        service_entries.extend(global_entries);

        Ok(service_entries)
    }
}

impl Profile {
    pub async fn show(
        instance: &ProfileInstance,
        id: u64,
        entries: Vec<String>,
    ) -> Result<BTreeMap<String, String>, mongodb::error::Error> {
        Self::get_int(instance, id, entries).await
    }

    pub async fn show_service(
        instance: &ProfileInstance,
        id: u64,
        service: &str,
        entries: Vec<String>,
    ) -> Result<BTreeMap<String, String>, mongodb::error::Error> {
        Self::get_service_int(instance, id, service, entries).await
    }

    pub async fn show_overlay(
        instance: &ProfileInstance,
        id: u64,
        service: &str,
        entries: Vec<String>,
    ) -> Result<BTreeMap<String, String>, mongodb::error::Error> {
        Self::get_overlay_int(instance, id, service, entries).await
    }

    pub async fn set(
        instance: &ProfileInstance,
        id: u64,
        entries: Vec<(String, String)>,
    ) -> Result<(), mongodb::error::Error> {
        Self::set_int(instance, id, entries).await
    }

    pub async fn set_service(
        instance: &ProfileInstance,
        id: u64,
        service: &str,
        entries: Vec<(String, String)>,
    ) -> Result<(), mongodb::error::Error> {
        Self::set_service_int(instance, id, service, entries).await
    }

    pub async fn remove(instance: &ProfileInstance, id: u64) -> Result<(), mongodb::error::Error> {
        Self::remove_int(instance, id).await
    }

    pub async fn remove_service(
        instance: &ProfileInstance,
        id: u64,
        service: &str,
    ) -> Result<(), mongodb::error::Error> {
        Self::remove_service_int(instance, id, service).await
    }
}
