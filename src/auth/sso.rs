//! EVE SSO (OAuth2), as the Socialite EVE Online provider and the
//! `nicolaskion/eve` connector used it: the authorize redirect with the
//! scopes a link asks for, the code traded for tokens with basic auth (the
//! access token is a JWT, verified against EVE's published keys: signature,
//! expiry, issuer), and the refresh of a stored token.

use std::fmt;
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use crate::config::EveConfig;

/// The issuers EVE's tokens carry.
const ACCEPTED_ISSUERS: [&str; 2] = ["login.eveonline.com", "https://login.eveonline.com"];

/// The prefix of the `sub` claim in front of the character id.
const SUBJECT_PREFIX: &str = "CHARACTER:EVE:";

/// Tries of a refresh EVE answers with a server error.
const REFRESH_TRIES: usize = 5;

#[derive(Debug)]
pub enum SsoError {
    Http(reqwest::Error),
    Status(reqwest::StatusCode),
    Token(jsonwebtoken::errors::Error),
    Claims(&'static str),
}

impl fmt::Display for SsoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SsoError::Http(error) => write!(f, "SSO request failed: {error}"),
            SsoError::Status(status) => write!(f, "unexpected SSO status: {status}"),
            SsoError::Token(error) => write!(f, "invalid SSO token: {error}"),
            SsoError::Claims(what) => write!(f, "malformed SSO token: {what}"),
        }
    }
}

impl std::error::Error for SsoError {}

impl From<reqwest::Error> for SsoError {
    fn from(error: reqwest::Error) -> Self {
        SsoError::Http(error)
    }
}

impl From<jsonwebtoken::errors::Error> for SsoError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        SsoError::Token(error)
    }
}

/// A login: the character behind the verified token and the tokens to
/// store.
#[derive(Debug, Clone)]
pub struct Login {
    pub character_id: i64,
    pub character_name: String,
    pub owner_hash: String,
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub scopes: Vec<String>,
}

/// New tokens from a refresh.
#[derive(Debug, Clone, Deserialize)]
pub struct Refreshed {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(default)]
    pub expires_in: Option<i64>,
}

#[derive(Debug)]
pub enum RefreshError {
    /// EVE answered with an error status (after the tries for a server
    /// error): the refresh token is dead.
    Rejected(u16),
    /// EVE could not be reached; the token is left alone.
    Network(String),
}

impl fmt::Display for RefreshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefreshError::Rejected(status) => write!(f, "refresh rejected with {status}"),
            RefreshError::Network(error) => write!(f, "SSO unreachable: {error}"),
        }
    }
}

#[derive(Deserialize)]
struct TokenAnswer {
    access_token: String,
    refresh_token: String,
    #[serde(default)]
    token_type: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
}

#[derive(Deserialize)]
struct Claims {
    sub: String,
    name: String,
    owner: String,
    #[serde(default)]
    scp: Option<Value>,
}

#[derive(Clone)]
pub struct SsoClient {
    config: EveConfig,
    http: reqwest::Client,
}

/// `http_build_query`'s encoding: RFC 1738, a space is `+`.
pub fn form_encode(value: &str) -> String {
    form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

impl SsoClient {
    pub fn new(config: EveConfig, user_agent: &str) -> Self {
        Self {
            config,
            http: reqwest::Client::builder()
                .user_agent(user_agent)
                .timeout(Duration::from_secs(30))
                .build()
                .expect("an HTTP client"),
        }
    }

    /// The authorize URL with Socialite's parameters in its order: the
    /// scopes (space separated, none for a plain login), the callback
    /// registered with CCP and the state.
    pub fn authorize_url(&self, scopes: &[String], state: &str) -> String {
        format!(
            "{}/v2/oauth/authorize/?client_id={}&redirect_uri={}&scope={}&response_type=code&state={}",
            self.config.sso_base_url,
            form_encode(&self.config.client_id),
            form_encode(&self.config.callback_url),
            form_encode(&scopes.join(" ")),
            form_encode(state),
        )
    }

    /// Trades the authorization code for tokens and reads the character
    /// and its scopes from the access token.
    pub async fn login(&self, code: &str) -> Result<Login, SsoError> {
        let response = self
            .http
            .post(format!("{}/v2/oauth/token", self.config.sso_base_url))
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .form(&[("grant_type", "authorization_code"), ("code", code)])
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(SsoError::Status(response.status()));
        }
        let answer: TokenAnswer = response.json().await?;
        let claims = self.verify(&answer.access_token).await?;
        let character_id = claims
            .sub
            .strip_prefix(SUBJECT_PREFIX)
            .and_then(|id| id.parse().ok())
            .ok_or(SsoError::Claims("the subject is no character"))?;
        let scopes = match claims.scp {
            Some(Value::String(scope)) => vec![scope],
            Some(Value::Array(scopes)) => scopes
                .into_iter()
                .filter_map(|scope| scope.as_str().map(str::to_owned))
                .collect(),
            _ => Vec::new(),
        };
        Ok(Login {
            character_id,
            character_name: claims.name,
            owner_hash: claims.owner,
            access_token: answer.access_token,
            refresh_token: answer.refresh_token,
            token_type: answer.token_type.unwrap_or_else(|| "Bearer".to_owned()),
            expires_in: answer.expires_in.unwrap_or(0),
            scopes,
        })
    }

    /// Verifies a token against the JWKS: the key named in its header, the
    /// algorithm that key carries (RS256 or ES256, never the token's own
    /// claim), its expiry and issuer. The audience is the app's client id;
    /// the provider did not check it either.
    async fn verify(&self, token: &str) -> Result<Claims, SsoError> {
        let response = self
            .http
            .get(format!("{}/oauth/jwks", self.config.sso_base_url))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(SsoError::Status(response.status()));
        }
        let jwks: jsonwebtoken::jwk::JwkSet = response.json().await?;
        let header = jsonwebtoken::decode_header(token)?;
        let key = match &header.kid {
            Some(kid) => jwks.find(kid),
            None => jwks.keys.first(),
        }
        .ok_or(SsoError::Claims("no matching key"))?;
        let algorithm = match &key.algorithm {
            jsonwebtoken::jwk::AlgorithmParameters::RSA(_) => jsonwebtoken::Algorithm::RS256,
            jsonwebtoken::jwk::AlgorithmParameters::EllipticCurve(_) => {
                jsonwebtoken::Algorithm::ES256
            }
            _ => return Err(SsoError::Claims("unsupported key type")),
        };
        let mut validation = jsonwebtoken::Validation::new(algorithm);
        validation.set_issuer(&ACCEPTED_ISSUERS);
        validation.validate_aud = false;
        Ok(jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_jwk(key)?,
            &validation,
        )?
        .claims)
    }

    /// Refreshes a stored token: tried five times while EVE answers with a
    /// server error, a second apart; any other failed status, or a server
    /// error on the last try, is a rejection.
    pub async fn refresh(&self, refresh_token: &str) -> Result<Refreshed, RefreshError> {
        let mut status = 0;
        for attempt in 1..=REFRESH_TRIES {
            let response = self
                .http
                .post(format!("{}/v2/oauth/token", self.config.sso_base_url))
                .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
                .form(&[
                    ("grant_type", "refresh_token"),
                    ("refresh_token", refresh_token),
                ])
                .send()
                .await
                .map_err(|error| RefreshError::Network(error.to_string()))?;
            status = response.status().as_u16();
            if response.status().is_success() {
                return response
                    .json()
                    .await
                    .map_err(|error| RefreshError::Network(error.to_string()));
            }
            if status < 500 {
                break;
            }
            if attempt < REFRESH_TRIES {
                tokio::time::sleep(self.config.refresh_retry_delay).await;
            }
        }
        Err(RefreshError::Rejected(status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_socialites_authorize_url() {
        let client = SsoClient::new(
            EveConfig {
                sso_base_url: "https://login.eveonline.com".to_owned(),
                client_id: "client id".to_owned(),
                client_secret: "secret".to_owned(),
                callback_url: "https://bluebook.nicolaskion.dev/eve/callback".to_owned(),
                refresh_retry_delay: Duration::ZERO,
            },
            "test",
        );
        assert_eq!(
            client.authorize_url(&[], "abc"),
            "https://login.eveonline.com/v2/oauth/authorize/?client_id=client+id&redirect_uri=https%3A%2F%2Fbluebook.nicolaskion.dev%2Feve%2Fcallback&scope=&response_type=code&state=abc"
        );
        assert!(
            client
                .authorize_url(&["a.v1".to_owned(), "b.v1".to_owned()], "abc")
                .contains("&scope=a.v1+b.v1&")
        );
    }
}
