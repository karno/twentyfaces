// this code derived from https://qiita.com/hppRC/items/05a81b56d12d663d03e0

use base64;
use chrono::Utc;
use hmac::{Hmac, Mac};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use reqwest::header::*;
use sha1::{Digest, Sha1};
use std::collections::HashMap;

type HmacSha1 = Hmac<Sha1>;

// twitter api endpoints
const TAE_REQUEST_TOKEN: &str = "https://api.twitter.com/oauth/request_token";

const FRAGMENT: &AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'*')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_');

#[derive(Clone, Debug)]
struct ConsumerKeySecret {
    consumer_key: String,
    consumer_secret: String,
}

#[derive(Clone, Debug)]
struct AccessToken {
    token: String,
    secret: String,
}

fn encode(input: &str) -> percent_encoding::PercentEncode {
    utf8_percent_encode(input, FRAGMENT)
}

fn create_oauth_signature(
    http_method: &str,
    endpoint: &str,
    consumer_secret: &str,
    token_secret: &str,
    params: &HashMap<&str, &str>,
) -> String {
    let key: String = format!("{}&{}", encode(consumer_secret), encode(token_secret));
    let mut params: Vec<(&&str, &&str)> = params.into_iter().collect();
    params.sort();
    let param = params
        .into_iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect::<Vec<String>>()
        .join("&");
    let payload = format!(
        "{}&{}&{}",
        encode(http_method),
        encode(endpoint),
        encode(&param)
    );
    let mut mac = HmacSha1::new_varkey(key.as_bytes()).expect("any size of keys can be accepted.");
    mac.input(payload.as_bytes());
    let hash = mac.result().code();
    base64::encode(&hash)
}

fn check_oauth_sig_gen(){
    create_oauth_signature("POST", "https://photos.example.net/initiate",
    ""
     consumer_secret: &str, token_secret: &str, params: &HashMap<&str, &str>)
}

fn get_request_header(endpoint: &str) {}
