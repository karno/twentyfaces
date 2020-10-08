use crate::{
    errors::ConfigurationError, errors::Error, twitter_api::misc::check_user_auth,
    twitter_api::TwitterResult,
};

use super::twitter_api;
use reqwest_oauth1::Secrets;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::{collections::hash_map::Entry, io};
use std::{collections::HashMap, fs::File};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO failed: {0}")]
    Io(#[from] io::Error),
    #[error("YAML parse failed: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Twitter API failed: {0}")]
    Twitter(#[from] twitter_api::TwitterError),
    #[error("User cancelled the action.")]
    UserCancelled,
}

pub trait SaveAndLoad: Sized + Serialize + DeserializeOwned {
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_yaml::to_writer(writer, self)?;
        Ok(())
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let payload = serde_yaml::from_reader(reader)?;
        Ok(payload)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ApiKey {
    pub consumer_key: String,
    pub consumer_secret: String,
}

impl SaveAndLoad for ApiKey {}

impl ApiKey {
    pub fn new(consumer_key: impl Into<String>, consumer_secret: impl Into<String>) -> ApiKey {
        ApiKey {
            consumer_key: consumer_key.into(),
            consumer_secret: consumer_secret.into(),
        }
    }

    pub fn as_secrets<'a>(&'a self) -> reqwest_oauth1::Secrets<'a> {
        Secrets::new(self.consumer_key.clone(), self.consumer_secret.clone())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    auth_info: AuthInfo,
    property: Property,
    profiles: Vec<Profile>,
}

impl SaveAndLoad for Config {}

impl Config {
    pub fn new(auth_info: AuthInfo, property: Property, profiles: &[Profile]) -> Config {
        Config {
            auth_info,
            property,
            profiles: profiles.to_vec(),
        }
    }

    pub fn new_example(auth_info: AuthInfo) -> Config {
        Config {
            auth_info,
            property: Property::create_sample(),
            profiles: vec![Profile::create_sample()],
        }
    }

    pub fn auth_info(&self) -> &AuthInfo {
        &self.auth_info
    }

    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    pub fn property(&self) -> &Property {
        &self.property
    }

    pub async fn validate(&mut self, api_key: &ApiKey) -> Result<(), Error> {
        // check authentication information
        let _ = check_user_auth(api_key, self.auth_info()).await?;
        // check profiles
        let mut map = HashMap::new();

        // 1. check all of the profiles has each unique keys
        for profile in self.profiles() {
            match map.entry(profile.key.as_str()) {
                Entry::Vacant(e) => {
                    e.insert(profile);
                    Ok(())
                }
                Entry::Occupied(_) => Err(ConfigurationError::new(format!(
                    "Duplicated key has been detected: {}",
                    profile.key,
                ))),
            }?;
        }

        // 2. check the derived profile keys are valid
        for mut profile in self.profiles() {
            loop {
                match &profile.derive {
                    Some(p) => {
                        // resolve profile
                        profile = map.get(p.as_str()).ok_or_else(|| {
                            ConfigurationError::new(format!(
                                "Unknown profile key {} is specified as derive in profile {}",
                                profile.key, p
                            ))
                        })?;
                    }
                    None => break,
                }
            }
        }

        // 3. check the regex patterns in the profile is valid
        for mut p in self.profiles.iter_mut() {
            p.match_instances = p
                .matches
                .iter()
                .map(|p| regex::Regex::new(p))
                .collect::<Result<Vec<regex::Regex>, regex::Error>>()
                .map_err(|e| ConfigurationError::new(format!("invalid regex pattern: {:?}", e)))?;
        }

        Ok(())
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Default, Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    pub user_id: u64,
    pub token: String,
    pub secret: String,
}

impl AuthInfo {
    pub fn new(user_id: u64, token: impl Into<String>, secret: impl Into<String>) -> AuthInfo {
        AuthInfo {
            user_id: user_id,
            token: token.into(),
            secret: secret.into(),
        }
    }
}

pub trait AuthInfoConfigurer<'a> {
    fn auth_info(self, auth_info: &'a AuthInfo) -> Secrets<'a>;

    fn auth_info_option(self, auth_info: Option<&'a AuthInfo>) -> Secrets<'a>
    where
        Self: Sized + Into<Secrets<'a>>,
    {
        match auth_info {
            Some(auth_info) => self.auth_info(auth_info),
            None => self.into(),
        }
    }
}

impl<'a> AuthInfoConfigurer<'a> for Secrets<'a> {
    fn auth_info(self, auth_info: &'a AuthInfo) -> Secrets<'a> {
        self.token(auth_info.token.clone(), auth_info.secret.clone())
    }
}

#[derive(Clone, Hash, Default, Debug, Serialize, Deserialize)]
pub struct Property {
    pub trigger_retweet: bool,
    pub trigger_quote: bool,
    pub trigger_reply: bool,
    pub trigger_direct_messages: bool,
}

impl Property {
    pub fn create_sample() -> Self {
        Property {
            trigger_retweet: false,
            trigger_quote: false,
            trigger_reply: false,
            trigger_direct_messages: false,
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub key: String,
    pub matches: Vec<String>,
    pub triggers: Vec<String>,
    pub derive: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub banner: Option<String>,
    pub intro: Option<String>,
    #[serde(skip)]
    pub match_instances: Vec<regex::Regex>,
}

#[derive(Clone, Debug, Default)]
pub struct ResolvedProfile<'a> {
    pub derive: Option<&'a String>,
    pub name: Option<&'a String>,
    pub url: Option<&'a String>,
    pub location: Option<&'a String>,
    pub description: Option<&'a String>,
    pub image: Option<&'a String>,
    pub banner: Option<&'a String>,
    pub intro: Option<&'a String>,
}

impl Profile {
    pub fn create_sample() -> Profile {
        Profile {
            key: "sample".to_string(),
            matches: vec![".*change.*sample".to_string()],
            triggers: vec!["sample".to_string()],
            derive: None,
            name: Some("sample profile".to_string()),
            url: Some("example.com".to_string()),
            location: Some("sample location".to_string()),
            description: Some("this is sample profile.".to_string()),
            image: Some("~/profile_image.png".to_string()),
            banner: Some("~/banner_image.png".to_string()),
            intro: Some("Hello, I'm a example profile!".to_string()),
            match_instances: vec![regex::Regex::new(".*change.*sample").unwrap()],
        }
    }

    pub fn resolve<'a>(
        &self,
        profiles: &'a [Profile],
    ) -> Result<ResolvedProfile<'a>, ConfigurationError> {
        // resolve derived profile
        let mut resolved: ResolvedProfile = Default::default();
        let mut key = Some(self.key.as_str());
        loop {
            // find deriving profile
            let derived = key.and_then(|k| profiles.iter().filter(|p| p.key == k).next());
            if let Some(Profile {
                name,
                url,
                location,
                description,
                image,
                banner,
                intro,
                derive,
                ..
            }) = derived
            {
                // replace empty value with derived item's

                resolved.name = resolved.name.or(name.as_ref());
                resolved.url = resolved.url.or(url.as_ref());
                resolved.location = resolved.location.or(location.as_ref());
                resolved.description = resolved.description.or(description.as_ref());
                resolved.image = resolved.image.or(image.as_ref());
                resolved.banner = resolved.banner.or(banner.as_ref());
                resolved.intro = resolved.intro.or(intro.as_ref());
                // scan next derived key
                key = derive.as_ref().map(|s| s.as_str());
            } else {
                break;
            }
        }
        Ok(resolved)
    }
}

impl ResolvedProfile<'_> {
    pub async fn apply(&self, api_key: &ApiKey, config: &Config) -> TwitterResult<()> {
        let auth_info = config.auth_info();

        Ok(())
    }
}
