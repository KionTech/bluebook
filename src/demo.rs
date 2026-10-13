//! The local demo. Not ported yet.

use sqlx::PgPool;
use time::OffsetDateTime;

pub async fn seed(_pool: &PgPool, _now: OffsetDateTime, _character_id: i64) -> sqlx::Result<()> {
    todo!("the demo seed")
}
