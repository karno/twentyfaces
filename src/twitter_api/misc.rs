use super::*;
use crate::config::*;
use crate::twitter_api::Error;
use oauthsign;
use reqwest;
use reqwest::header;

pub async fn check_api_key(api_key: &ApiKey) -> Result<(), Error> {
    get_rate_limit_status_core(api_key, None).await
}

pub async fn check_user_auth(api_key: &ApiKey, user: &AuthInfo) -> Result<(), Error> {
    get_rate_limit_status_core(api_key, Some(user)).await
}

async fn get_rate_limit_status_core(
    api_key: &ApiKey,
    user: Option<&AuthInfo>,
) -> Result<(), Error> {
    let endpoint = "https://api.twitter.com/1.1/application/rate_limit_status.json";
    let sign_builder = oauthsign::v1::OAuthSignBuilder::new(&api_key.consumer_key);
    let sign = sign_builder.sign(endpoint, "GET", &api_key.consumer_secret);
    let client = reqwest::Client::new();
    let built = client
        .get(endpoint)
        .header(header::AUTHORIZATION, format!("OAuth {}", sign));

    println!("{:#?}", built);
    let resp = built.send().await?;
    println!("{:#?}", resp);
    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else {
        Err(Error::Http(status.as_u16(), String::from(status.as_str())))
    }
}
