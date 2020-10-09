use std::collections::HashMap;

use reqwest::multipart;
use reqwest_oauth1::OAuthClientProvider;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::config::{ApiKey, AuthInfo, AuthInfoConfigurer};

use super::{CheckSuccess, TwitterResult};

pub async fn update_profile_image(
    api_key: &ApiKey,
    user: &AuthInfo,
    image_path: &str,
) -> TwitterResult<()> {
    let endpoint = "https://api.twitter.com/1.1/account/update_profile_image.json";
    let secret = api_key.as_secrets().auth_info(user);
    // read file
    let file = tokio::fs::File::open(image_path).await?;
    let reader = FramedRead::new(file, BytesCodec::new());
    // payload is "image"
    let part = multipart::Part::stream(reqwest::Body::wrap_stream(reader));
    let form = multipart::Form::new().part("image", part);
    let resp = reqwest::Client::new()
        .oauth1(secret)
        .post(endpoint)
        .multipart(form)
        .send()
        .await?;
    resp.check_success().await?;
    Ok(())
}

pub async fn update_profile(
    api_key: &ApiKey,
    user: &AuthInfo,
    name: Option<&str>,
    url: Option<&str>,
    location: Option<&str>,
    description: Option<&str>,
) -> TwitterResult<()> {
    let endpoint = "https://api.twitter.com/1.1/account/update_profile.json";
    let secret = api_key.as_secrets().auth_info(user);
    let mut form = HashMap::new();

    if let Some(name) = name {
        form.insert("name", name);
    }

    if let Some(url) = url {
        form.insert("url", url);
    }
    if let Some(location) = location {
        form.insert("location", location);
    }
    if let Some(description) = description {
        form.insert("description", description);
    }

    let resp = reqwest::Client::new()
        .oauth1(secret)
        .post(endpoint)
        .form(&form)
        .send()
        .await?;
    resp.check_success().await?;
    Ok(())
}

pub async fn update_profile_banner(
    api_key: &ApiKey,
    user: &AuthInfo,
    image_path: &str,
) -> TwitterResult<()> {
    let endpoint = "https://api.twitter.com/1.1/account/update_profile_banner.json";
    let secret = api_key.as_secrets().auth_info(user);
    // read file
    let file = tokio::fs::File::open(image_path).await?;
    let reader = FramedRead::new(file, BytesCodec::new());
    let part = multipart::Part::stream(reqwest::Body::wrap_stream(reader));
    let form = multipart::Form::new().part("banner", part);
    let resp = reqwest::Client::new()
        .oauth1(secret)
        .post(endpoint)
        .multipart(form)
        .send()
        .await?;
    resp.check_success().await?;
    Ok(())
}
