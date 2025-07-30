use std::{num::NonZeroUsize, str::FromStr, sync::Arc};
use tokio::sync::Mutex;

use axum_login::tower_sessions::{
    SessionStore,
    session::{Id, Record},
    session_store::{self, Error as SessionStoreError},
};
use chrono::{DateTime, FixedOffset, Utc};
use lru::LruCache;
use sea_orm::{
    ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait,
    prelude::async_trait::async_trait,
};
use time::{UtcDateTime, UtcOffset};

use crate::generated::session;

#[derive(Clone, Debug)]
pub struct SeaSessionBackend {
    db: DatabaseConnection,
    lru: Arc<Mutex<LruCache<Id, Record>>>,
}

impl SeaSessionBackend {
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            lru: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap()))),
        }
    }
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
        let insert = new_session.insert(&self.db).await.map_err(|err| {
            log::error!("Failed to insert session: {err:?}");
            backend_error("Failed to insert session")
        });
        self.lru.lock().await.put(record.id, record.clone());
        insert.map(|_| ())
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
        self.lru.lock().await.put(record.id, record.clone());
        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        if let Some(record) = self.lru.lock().await.get(session_id) {
            log::debug!("Session found in cache: {session_id}");
            return Ok(Some(record.clone()));
        }
        let session = session::Entity::find_by_id(session_id.to_string())
            .one(&self.db)
            .await
            .map_err(|_| backend_error("Failed to load session"))?;
        let Some(sess) = session else {
            return Ok(None);
        };
        let Some(expiry_time) = sess.expires_at.timestamp_nanos_opt().map(i128::from) else {
            return Err(decode_error("Failed to decode expiry time"));
        };
        let expiry_date = match UtcDateTime::from_unix_timestamp_nanos(expiry_time) {
            Ok(date) => date.to_offset(UtcOffset::UTC),
            Err(_) => return Err(decode_error("Failed to convert expiry time")),
        };
        let Ok(id) = Id::from_str(&sess.id) else {
            return Err(decode_error("Failed to parse session ID"));
        };
        let Ok(data) = serde_json::from_value(sess.data) else {
            return Err(decode_error("Failed to decode session data"));
        };
        let record = Record {
            id,
            data,
            expiry_date,
        };
        self.lru.lock().await.put(*session_id, record.clone());
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
        self.lru.lock().await.pop(session_id);
        Ok(())
    }
}

fn backend_error(message: &str) -> SessionStoreError {
    SessionStoreError::Backend(message.to_owned())
}

fn decode_error(message: &str) -> SessionStoreError {
    SessionStoreError::Decode(message.to_owned())
}

fn convert_time(time: i128) -> session_store::Result<DateTime<FixedOffset>> {
    let nanos: i64 = time
        .try_into()
        .map_err(|_| SessionStoreError::Encode("Failed to convert expiry time".to_owned()))?;
    Ok(DateTime::from_timestamp_nanos(nanos).fixed_offset())
}

// async fn still_valid_auth(jar: &mut SeaAuthSession, state: &AppState) -> Option<Claims> {
//     let jwt = get_cookie_value("token", jar);
//     if jwt.is_empty() {
//         log::debug!("Missing auth cookie");
//         return None;
//     }
//     log::debug!("Cookie found, validating");
//     match decode::<Claims>(&jwt, &state.jwk, &build_validation()) {
//         Ok(session) => Some(session.claims),
//         Err(err) => {
//             log::debug!("Token did not pass validation {:?}", err);
//             if *err.kind() == ErrorKind::ExpiredSignature {
//                 log::debug!("JWT is expired, attempting to refresh");
//                 let refresh_token = get_cookie_value("refresh_token", jar);
//                 let payload = from_refresh_to_token_payload(refresh_token);
//                 let refreshed = exchange_token(state, &Either::Right(payload))
//                     .await
//                     .inspect_err(|err| log::error!("Failed to refresh token: '{:?}'", err))
//                     .ok()?;
//                 log::debug!("Successfully refreshed, persisting the new token");
//                 *jar = jar
//                     .clone()
//                     .add(safe_cookie("token", &refreshed.access_token));
//                 let decoded =
//                     decode::<Claims>(&refreshed.access_token, &state.jwk, &build_validation());
//                 return decoded
//                     .inspect_err(|err| {
//                         log::error!(
//                             "Failed decoding JWT: {:?} (jwt={})",
//                             err,
//                             &refreshed.access_token
//                         );
//                     })
//                     .map(|x| x.claims)
//                     .ok();
//             } else if *err.kind() == ErrorKind::InvalidAudience {
//                 log::debug!("Invalid audience JWT: {jwt}");
//             }
//             None
//         }
//     }
// }
