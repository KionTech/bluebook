//! Who is asking and what EVE lets the app do: the session cookie
//! (`session`), EVE SSO (`sso`), the login (`login`) and the stored ESI
//! tokens (`tokens`).

pub mod login;
pub mod session;
pub mod sso;
pub mod tokens;
