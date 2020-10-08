use std::borrow::Cow;

use reqwest_oauth1::{OAuthClientProvider, TokenReaderFuture, TokenResponse};

use crate::config::ApiKey;

// twitter api endpoints
const EP_REQUEST_TOKEN: &str = "https://api.twitter.com/oauth/request_token";
const EP_AUTHORIZE_FORMAT: &str = "https://api.twitter.com/oauth/authorize?oauth_token=";
const EP_ACCESS_TOKEN: &str = "https://api.twitter.com/oauth/access_token";

pub async fn request_token(api_key: &ApiKey) -> super::TwitterResult<TokenResponse> {
    let resp = reqwest::Client::new()
        .oauth1(api_key.as_secrets())
        .post(EP_REQUEST_TOKEN)
        .query(&[("oauth_callback", "oob")])
        .send()
        .parse_oauth_token()
        .await?;
    Ok(resp)
}

pub fn get_authorization_url(resp: &TokenResponse) -> String {
    format!("{}{}", EP_AUTHORIZE_FORMAT, resp.oauth_token)
}

pub async fn access_token<'a, T: Into<Cow<'a, str>>>(
    api_key: &ApiKey,
    req_token: TokenResponse,
    pin: T,
) -> super::TwitterResult<TokenResponse> {
    let resp = reqwest::Client::new()
        .oauth1(
            api_key
                .as_secrets()
                .token(req_token.oauth_token, req_token.oauth_token_secret),
        )
        .post(EP_ACCESS_TOKEN)
        .query(&[("oauth_verifier", pin.into())])
        .send()
        .parse_oauth_token()
        .await?;
    Ok(resp)
}
