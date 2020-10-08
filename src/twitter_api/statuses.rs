use reqwest_oauth1::OAuthClientProvider;

use crate::config::{ApiKey, AuthInfo, AuthInfoConfigurer};

use super::models::Status;
use super::TwitterResult;

pub async fn home_timeline(
    api_key: &ApiKey,
    user: &AuthInfo,
    count: Option<u32>,
) -> TwitterResult<Vec<Status>> {
    let endpoint = "https://api.twitter.com/1.1/statuses/home_timeline.json";
    let secret = api_key.as_secrets().auth_info(user);

    let resp = reqwest::Client::new()
        .oauth1(secret)
        .get(endpoint)
        .query(&[("count", count.unwrap_or(20u32))])
        .send()
        .await?;
    let body = resp.error_for_status()?.text().await?;
    Ok(Status::deserialize_timeline(&body)?)
}

pub async fn user_timeline(
    api_key: &ApiKey,
    user: &AuthInfo,
    count: Option<u32>,
) -> TwitterResult<Vec<Status>> {
    let endpoint = "https://api.twitter.com/1.1/statuses/user_timeline.json";
    let secret = api_key.as_secrets().auth_info(user);

    let resp = reqwest::Client::new()
        .oauth1(secret)
        .get(endpoint)
        .query(&[("count", count.unwrap_or(20u32))])
        .send()
        .await?;
    let body = resp.error_for_status()?.text().await?;
    Ok(Status::deserialize_timeline(&body)?)
}
