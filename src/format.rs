//! The wire formats of the page props: standings without a needless
//! fraction and ISO 8601 times with an explicit offset.

use serde_json::{Value, json};
use time::OffsetDateTime;
use time::macros::format_description;

/// A standing in JSON: a whole number without its fraction (`10`, `-5`),
/// anything else as it is (`7.5`).
pub fn standing_value(value: f64) -> Value {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        json!(value as i64)
    } else {
        json!(value)
    }
}

/// A standing as text (`5`, `-10`, `7.5`), the form the contact writes
/// put into their query.
pub fn standing_string(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

/// A time in UTC as ISO 8601: `2026-07-24T12:18:44+00:00`.
pub fn iso8601(moment: OffsetDateTime) -> String {
    let utc = moment.to_offset(time::UtcOffset::UTC);
    utc.format(format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second]+00:00"
    ))
    .expect("a formatted time")
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn prints_standings_without_a_needless_fraction() {
        assert_eq!(standing_value(10.0), json!(10));
        assert_eq!(standing_value(-5.0), json!(-5));
        assert_eq!(standing_value(0.0), json!(0));
        assert_eq!(standing_value(7.5), json!(7.5));
        assert_eq!(standing_string(5.0), "5");
        assert_eq!(standing_string(-10.0), "-10");
        assert_eq!(standing_string(7.5), "7.5");
        assert_eq!(standing_string(-0.25), "-0.25");
    }

    #[test]
    fn formats_times_with_an_explicit_offset() {
        assert_eq!(
            iso8601(datetime!(2026-07-24 12:18:44 UTC)),
            "2026-07-24T12:18:44+00:00"
        );
    }
}
