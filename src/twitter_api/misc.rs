use super::*;
use crate::config::*;
use reqwest;
use reqwest_oauth1::OAuthClientProvider;
// use reqwest_oauth1::client::OAuthClientProvider;

pub async fn check_api_key(api_key: &ApiKey) -> TwitterResult<()> {
    get_rate_limit_status_core(api_key, None).await
}

pub async fn check_user_auth(api_key: &ApiKey, user: &AuthInfo) -> TwitterResult<()> {
    get_rate_limit_status_core(api_key, Some(user)).await
}

async fn get_rate_limit_status_core(
    api_key: &ApiKey,
    user: Option<&AuthInfo>,
) -> TwitterResult<()> {
    let endpoint = "https://api.twitter.com/1.1/application/rate_limit_status.json";
    let secret = api_key.as_secrets().auth_info_option(user);

    let resp = reqwest::Client::new()
        .oauth1(secret)
        .get(endpoint)
        .send()
        .await?;
    resp.error_for_status()?;
    Ok(())
}
