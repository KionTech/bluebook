//! A form's fields: the JSON object the pages send (or a urlencoded form),
//! where an empty string is null.

use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::header;
use axum::response::Response;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Default)]
pub struct Input(pub Map<String, Value>);

impl Input {
    pub fn from_bytes(content_type: &str, body: &[u8]) -> Self {
        let fields = if content_type.starts_with("application/x-www-form-urlencoded") {
            form_urlencoded::parse(body)
                .map(|(key, value)| (key.into_owned(), Value::String(value.into_owned())))
                .collect()
        } else {
            serde_json::from_slice::<Value>(body)
                .ok()
                .and_then(|value| value.as_object().cloned())
                .unwrap_or_default()
        };
        Self(
            fields
                .into_iter()
                .map(|(key, value)| match value {
                    Value::String(text) if text.is_empty() => (key, Value::Null),
                    other => (key, other),
                })
                .collect(),
        )
    }

    /// A field; missing and null are the same.
    pub fn get(&self, field: &str) -> Option<&Value> {
        self.0.get(field).filter(|value| !value.is_null())
    }

    pub fn str(&self, field: &str) -> Option<&str> {
        self.get(field).and_then(Value::as_str)
    }
}

impl<S: Send + Sync> FromRequest<S> for Input {
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Response> {
        let content_type = request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        let body = Bytes::from_request(request, state)
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(Self::from_bytes(&content_type, &body))
    }
}

/// A boolean field: true, false, 1, 0, "1" or "0"; "true" and "false" as
/// strings are refused.
pub fn as_boolean(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(flag) => Some(*flag),
        Value::Number(number) => match number.as_i64() {
            Some(1) => Some(true),
            Some(0) => Some(false),
            _ => None,
        },
        Value::String(text) => match text.as_str() {
            "1" => Some(true),
            "0" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// The query string of a request: the parameters in order, the last of a
/// repeated name winning.
pub fn query_pairs(query: Option<&str>) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    for (key, value) in form_urlencoded::parse(query.unwrap_or_default().as_bytes()) {
        match pairs.iter_mut().find(|(existing, _)| *existing == key) {
            Some(pair) => pair.1 = value.into_owned(),
            None => pairs.push((key.into_owned(), value.into_owned())),
        }
    }
    pairs
}

pub fn query_value(query: Option<&str>, name: &str) -> Option<String> {
    query_pairs(query)
        .into_iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_strings_are_null() {
        let input = Input::from_bytes("application/json", br#"{"a":"","b":"x","c":null}"#);
        assert_eq!(input.get("a"), None);
        assert_eq!(input.str("b"), Some("x"));
        assert_eq!(input.get("c"), None);
        let form = Input::from_bytes("application/x-www-form-urlencoded", b"a=&b=y");
        assert_eq!(form.get("a"), None);
        assert_eq!(form.str("b"), Some("y"));
    }

    #[test]
    fn reads_booleans() {
        assert_eq!(as_boolean(&json!(true)), Some(true));
        assert_eq!(as_boolean(&json!(0)), Some(false));
        assert_eq!(as_boolean(&json!("1")), Some(true));
        assert_eq!(as_boolean(&json!("maybe")), None);
        assert_eq!(as_boolean(&json!("true")), None);
        assert_eq!(as_boolean(&json!(2)), None);
    }

    #[test]
    fn reads_the_query_in_order() {
        assert_eq!(
            query_pairs(Some("view=a&search=%20x%20&view=b")),
            vec![
                ("view".to_owned(), "b".to_owned()),
                ("search".to_owned(), " x ".to_owned())
            ]
        );
    }
}
