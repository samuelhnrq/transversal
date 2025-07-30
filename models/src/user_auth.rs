use axum_login::{AuthUser, AuthnBackend};
use sea_orm::prelude::*;
use std::io::Error as IOError;

use crate::{
    generated::user,
    oauth::{TokenResponse, UserInfo},
    state::AppState,
    user_repository::{get_user_by_id, upsert_user},
};

impl AuthUser for user::Model {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.sid.as_bytes()
    }
}

#[derive(Clone)]
pub struct SeaAuthBackend {
    state: AppState,
}

impl SeaAuthBackend {
    #[must_use]
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl AuthnBackend for SeaAuthBackend {
    type Credentials = TokenResponse;
    type User = user::Model;
    type Error = IOError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        log::debug!("Authenticating user with credentials");
        let resp = self
            .state
            .requests
            .get(&self.state.config.oauth.userinfo_endpoint)
            .bearer_auth(creds.access_token)
            .send()
            .await
            .map_err(IOError::other)?
            .json::<UserInfo>()
            .await
            .map_err(IOError::other)?;
        log::debug!("User data fetched successfully");
        let user = upsert_user(&self.state.db, resp)
            .await
            .inspect_err(|err| log::error!("Failed to upsert user: {err}"))?;
        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &Uuid) -> Result<Option<Self::User>, Self::Error> {
        get_user_by_id(&self.state.db, user_id).await
    }
}

pub type SeaAuthSession = axum_login::AuthSession<SeaAuthBackend>;
