use chrono::{DateTime, Utc};
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use usual::{
    base::{Model, TryGetRow},
    UsualModel,
};
use usual_macros::petelib;
use uuid::Uuid;

#[petelib(create, read, update, destroy)]
#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    #[petelib(readonly, id)]
    pub(crate) id: Uuid,
    #[petelib(index)]
    pub(crate) email: String,
    #[petelib(secure)]
    pub(crate) password_hash: String,
    #[petelib(readonly)]
    pub(crate) created_at: DateTime<Utc>,
}

#[petelib(create, read, update, destroy)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Image {
    #[petelib(readonly, id)]
    pub id: Uuid,
    #[petelib(queryable)]
    pub user_id: Uuid,
    nickname: String,
    pub image_url: String,
    #[petelib(readonly)]
    created_at: DateTime<Utc>,
}

#[petelib(create, read, readall, destroy)]
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageVersion {
    #[petelib(readonly, id)]
    pub id: Uuid,
    #[petelib(queryable)]
    image_id: Uuid,
    hash: String,
    pub version_number: String,
    #[petelib(readonly)]
    created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSql, FromSql)]
pub enum JobStatus {
    Completed,
    Failed,
    Pending,
}

#[petelib(create, read, update, destroy)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Job {
    #[petelib(readonly, id)]
    id: Uuid,
    #[petelib(queryable)]
    user_id: Uuid,
    status: JobStatus,
    image_version_id: Uuid,
    #[petelib(readonly)]
    created_at: DateTime<Utc>,
    #[petelib(readonly)]
    updated_at: DateTime<Utc>,
}
