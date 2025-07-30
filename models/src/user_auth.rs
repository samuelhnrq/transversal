use std::io::Error as IOError;

use crate::{
    generated::user,
    oauth::{TokenResponse, UserInfo},
    repositories::user::{get_user_by_sub, upsert_user},
    state::AppState,
};

pub async fn authenticate(
    state: &AppState,
    creds: TokenResponse,
) -> Result<Option<user::Model>, IOError> {
    log::debug!("Authenticating user with credentials");
    let resp = state
        .requests
        .get(&state.config.oauth.userinfo_endpoint)
        .bearer_auth(creds.access_token)
        .send()
        .await
        .map_err(IOError::other)?
        .json::<UserInfo>()
        .await
        .map_err(IOError::other)?;
    log::debug!("User data fetched successfully");

    if let Ok(Some(user)) = get_user_by_sub(&state.db, &resp.sub).await {
        log::debug!("User found in database");
        return Ok(Some(user));
    }
    let user = upsert_user(&state.db, resp)
        .await
        .inspect_err(|err| log::error!("Failed to upsert user: {err}"))?;
    Ok(Some(user))
}
