//! EVE's daily downtime, 11:00 to 11:15 UTC (start included, end not):
//! ESI is down or unstable then, so the scheduled syncs skip their run.

use time::{OffsetDateTime, Time};

const START: Time = time::macros::time!(11:00);
const END: Time = time::macros::time!(11:15);

pub fn is_active(at: OffsetDateTime) -> bool {
    let time = at.to_offset(time::UtcOffset::UTC).time();
    time >= START && time < END
}
