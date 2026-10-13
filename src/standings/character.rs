//! The character sync. Not ported yet.

use crate::server::AppState;

pub async fn sync(_state: &AppState, _character_id: i64) -> sqlx::Result<()> {
    todo!("the character sync")
}
