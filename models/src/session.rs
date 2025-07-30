use std::str::FromStr;

use axum_login::tower_sessions::{
    SessionStore,
    session::{Id, Record},
    session_store::{self, Error as SessionStoreError},
};
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait,
    prelude::async_trait::async_trait,
};
use time::{UtcDateTime, UtcOffset};

use crate::generated::session;

#[derive(Clone, Debug)]
pub struct SeaSessionBackend {
    db: DatabaseConnection,
}

impl SeaSessionBackend {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn convert_time(time: i128) -> session_store::Result<DateTime<FixedOffset>> {
    let nanos: i64 = time
        .try_into()
        .map_err(|_| SessionStoreError::Encode("Failed to convert expiry time".to_owned()))?;
    Ok(DateTime::from_timestamp_nanos(nanos).fixed_offset())
}

#[async_trait]
impl SessionStore for SeaSessionBackend {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let expiry_time = record.expiry_date.unix_timestamp_nanos();
        let new_session = session::ActiveModel {
            id: ActiveValue::Set(record.id.to_string()),
            data: ActiveValue::Set(serde_json::to_value(record.data.clone()).unwrap()),
            expires_at: ActiveValue::Set(convert_time(expiry_time)?),
            ..Default::default()
        };
        new_session
            .insert(&self.db)
            .await
            .map_err(|_| backend_error("Failed to insert session"))?;
        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let expiry_time = record.expiry_date.unix_timestamp_nanos();
        let updated_session = session::ActiveModel {
            id: ActiveValue::Unchanged(record.id.to_string()),
            data: ActiveValue::Set(serde_json::to_value(record.data.clone()).unwrap()),
            expires_at: ActiveValue::Set(convert_time(expiry_time)?),
            refreshed_at: ActiveValue::Set(Utc::now().fixed_offset()),
            ..Default::default()
        };
        updated_session
            .update(&self.db)
            .await
            .map_err(|_| backend_error("Failed to update session"))?;
        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let session = session::Entity::find_by_id(session_id.to_string())
            .one(&self.db)
            .await
            .map_err(|_| backend_error("Failed to load session"))?;
        let Some(session) = session else {
            return Ok(None);
        };
        let expiry_time = session
            .expires_at
            .timestamp_nanos_opt()
            .ok_or_else(|| decode_error("Failed to get expiry date nanos"))?
            as i128;
        let expiry_date = UtcDateTime::from_unix_timestamp_nanos(expiry_time)
            .map_err(|_| decode_error("Failed to decode expiry date"))?
            .to_offset(UtcOffset::UTC);
        let record = Record {
            id: Id::from_str(&session.id)
                .map_err(|_| decode_error("Failed to parse session ID"))?,
            data: serde_json::from_value(session.data)
                .map_err(|_| decode_error("Failed to decode session data"))?,
            expiry_date,
        };
        Ok(Some(record))
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let session_record = session::ActiveModel {
            id: ActiveValue::Set(session_id.to_string()),
            ..Default::default()
        };
        session_record
            .delete(&self.db)
            .await
            .map_err(|_| backend_error("Failed to delete session"))?;
        Ok(())
    }
}

fn backend_error(message: &str) -> SessionStoreError {
    SessionStoreError::Backend(message.to_owned())
}

fn decode_error(message: &str) -> SessionStoreError {
    SessionStoreError::Decode(message.to_owned())
}
