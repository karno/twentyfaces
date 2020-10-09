use std::collections::HashMap;

use reqwest_oauth1::OAuthClientProvider;

use crate::config::{ApiKey, AuthInfo, AuthInfoConfigurer};

use super::TwitterResult;
use super::{models::Status, CheckSuccess};

pub async fn home_timeline(
    api_key: &ApiKey,
    user: &AuthInfo,
    count: Option<u32>,
    since_id: Option<u64>,
) -> TwitterResult<Vec<Status>> {
    let endpoint = "https://api.twitter.com/1.1/statuses/home_timeline.json";
    let secret = api_key.as_secrets().auth_info(user);
    let mut param = HashMap::new();
    if let Some(count) = count {
        param.insert("count", count as u64);
    }
    if let Some(since_id) = since_id {
        param.insert("since_id", since_id);
    }
    let resp = reqwest::Client::new()
        .oauth1(secret)
        .get(endpoint)
        .query(&param)
        .send()
        .await?;
    let body = resp.check_success().await?.text().await?;
    Ok(Status::deserialize_timeline(&body)?)
}

pub async fn user_timeline(
    api_key: &ApiKey,
    user: &AuthInfo,
    count: Option<u32>,
    since_id: Option<u64>,
) -> TwitterResult<Vec<Status>> {
    let endpoint = "https://api.twitter.com/1.1/statuses/user_timeline.json";
    let secret = api_key.as_secrets().auth_info(user);
    let mut param = HashMap::new();
    if let Some(count) = count {
        param.insert("count", count as u64);
    }
    if let Some(since_id) = since_id {
        param.insert("since_id", since_id);
    }

    let resp = reqwest::Client::new()
        .oauth1(secret)
        .get(endpoint)
        .query(&param)
        .send()
        .await?;
    let body = resp.check_success().await?.text().await?;
    Ok(Status::deserialize_timeline(&body)?)
}
