use super::{TwitterDataError, TwitterResult};
use serde_json::Value;

#[derive(Debug)]
pub enum StatusType {
    PublicStatus,
    DirectMessageTo(User),
}

#[derive(Debug)]
pub struct Status {
    pub status_type: StatusType,
    pub id: u64,
    pub user: User,
    pub text: String,
    pub in_reply_to_status_id: Option<u64>,
    pub retweeted_status: Option<Box<Status>>,
    pub quoted_status: Option<Box<Status>>,
}

#[derive(Debug)]
pub struct User {
    pub id: u64,
    pub screen_name: String,
    pub name: String,
    pub description: String,
    pub is_protected: bool,
}

impl Status {
    pub fn deserialize_timeline(json: &str) -> TwitterResult<Vec<Self>> {
        let v: Value = serde_json::from_str(json)?;
        match v {
            Value::Array(a) => a
                .into_iter()
                .map(|v| Status::deserialize_json_value(&v))
                .collect(),
            _ => Ok(vec![Status::deserialize_json_value(&v)?]),
        }
    }

    pub fn deserialize_json(json: &str) -> TwitterResult<Self> {
        let v: Value = serde_json::from_str(json)?;

        Status::deserialize_json_value(&v)
    }

    pub fn deserialize_json_value(value: &Value) -> TwitterResult<Self> {
        let status_type = match value.get("recipient").and_then(|r| r.as_str()) {
            Some(recipient) => StatusType::DirectMessageTo(User::deserialize_json(recipient)?),
            None => StatusType::PublicStatus,
        };
        let retweeted_status = match status_type {
            StatusType::PublicStatus => value
                .get("retweeted_status")
                .map(|v| Status::deserialize_json_value(v)),
            StatusType::DirectMessageTo(_) => None,
        }
        .map_or(Ok(None), |r| r.map(Box::new).map(Some))?;
        let quoted_status = match status_type {
            StatusType::PublicStatus => value
                .get("quoted_status")
                .map(|v| Status::deserialize_json_value(v)),
            StatusType::DirectMessageTo(_) => None,
        }
        .map_or(Ok(None), |r| r.map(Box::new).map(Some))?;
        let in_reply_to_status_id = value
            .get("in_reply_to_status_id")
            .and_then(|v| v.as_u64())
            // ensure non-zero value
            .and_then(|v| if v > 0 { Some(v) } else { None });

        Ok(Status {
            status_type,
            id: value.read_value("id")?,
            user: value
                .get("user")
                .map(User::deserialize_json_value)
                .ok_or_else(|| TwitterDataError::new("user", value.to_string()))??,
            text: value.read_value("text")?,
            in_reply_to_status_id,
            retweeted_status,
            quoted_status,
        })
    }
}

impl User {
    pub fn deserialize_json(json: &str) -> TwitterResult<Self> {
        let v: Value = serde_json::from_str(json)?;
        User::deserialize_json_value(&v)
    }

    pub fn deserialize_json_value(value: &Value) -> TwitterResult<Self> {
        Ok(User {
            id: value.read_value("id")?,
            screen_name: value.read_value("screen_name")?,
            name: value.read_value("name")?,
            description: value.read_value("description")?,
            is_protected: value.read_value("protected")?,
        })
    }
}

trait ReadValue<'a, T> {
    fn read_value(&'a self, key: &str) -> TwitterResult<T>;
}

impl ReadValue<'_, bool> for Value {
    fn read_value(&self, key: &str) -> TwitterResult<bool> {
        Ok(self[key]
            .as_bool()
            .ok_or_else(|| TwitterDataError::new(key, self.to_string()))?)
    }
}

impl<'a> ReadValue<'a, &'a str> for Value {
    fn read_value(&'a self, key: &str) -> TwitterResult<&'a str> {
        Ok(self[key]
            .as_str()
            .ok_or_else(|| TwitterDataError::new(key, self.to_string()))?)
    }
}

impl ReadValue<'_, String> for Value {
    fn read_value(&self, key: &str) -> TwitterResult<String> {
        Ok(self[key]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| TwitterDataError::new(key, self.to_string()))?)
    }
}

impl ReadValue<'_, u64> for Value {
    fn read_value(&self, key: &str) -> TwitterResult<u64> {
        Ok(self[key]
            .as_u64()
            .ok_or_else(|| TwitterDataError::new(key, self.to_string()))?)
    }
}
