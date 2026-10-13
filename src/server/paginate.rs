//! Laravel's `LengthAwarePaginator` as JSON, with `withQueryString()`: the
//! links keep the request's other query parameters and put `page` last.

use serde_json::{Value, json};

/// The page a request asks for: `?page=` when it is a whole number of at
/// least 1, else the first (Laravel's `resolveCurrentPage`).
pub fn current_page(query: &[(String, String)]) -> i64 {
    query
        .iter()
        .find(|(key, _)| key == "page")
        .and_then(|(_, value)| value.parse::<i64>().ok())
        .filter(|page| *page >= 1)
        .unwrap_or(1)
}

/// RFC 3986 encoding, as `http_build_query` with `PHP_QUERY_RFC3986`.
fn encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

pub struct Paginator<'a> {
    /// The page's absolute URL without its query.
    pub path: String,
    /// The request's query, `page` included or not.
    pub query: &'a [(String, String)],
    pub per_page: i64,
    pub current_page: i64,
    pub total: i64,
}

impl Paginator<'_> {
    pub fn last_page(&self) -> i64 {
        ((self.total + self.per_page - 1) / self.per_page).max(1)
    }

    pub fn offset(&self) -> i64 {
        (self.current_page - 1) * self.per_page
    }

    pub fn url(&self, page: i64) -> String {
        let mut parts: Vec<String> = self
            .query
            .iter()
            .filter(|(key, _)| key != "page")
            .map(|(key, value)| format!("{}={}", encode(key), encode(value)))
            .collect();
        parts.push(format!("page={page}"));
        format!("{}?{}", self.path, parts.join("&"))
    }

    /// The numbered links: every page up to 13, else Laravel's window of
    /// three pages around the current one and two at each end, with `...`
    /// between.
    fn numbered(&self) -> Vec<Option<i64>> {
        let last = self.last_page();
        let on_each_side = 3;
        if last < on_each_side * 2 + 8 {
            return (1..=last).map(Some).collect();
        }
        let window = on_each_side + 4;
        let mut items: Vec<Option<i64>> = Vec::new();
        let range = |from: i64, to: i64| (from..=to).map(Some).collect::<Vec<_>>();
        if self.current_page <= window {
            items.extend(range(1, window + on_each_side));
            items.push(None);
            items.extend(range(last - 1, last));
        } else if self.current_page > last - window {
            items.extend(range(1, 2));
            items.push(None);
            items.extend(range(last - (window + (on_each_side - 1)), last));
        } else {
            items.extend(range(1, 2));
            items.push(None);
            items.extend(range(
                self.current_page - on_each_side,
                self.current_page + on_each_side,
            ));
            items.push(None);
            items.extend(range(last - 1, last));
        }
        items
    }

    pub fn json(&self, data: Vec<Value>) -> Value {
        let last = self.last_page();
        let count = data.len() as i64;
        let (from, to) = if count == 0 {
            (Value::Null, Value::Null)
        } else {
            (json!(self.offset() + 1), json!(self.offset() + count))
        };
        let previous = (self.current_page > 1).then(|| self.current_page - 1);
        let next = (self.current_page < last).then(|| self.current_page + 1);
        let mut links = vec![json!({
            "url": previous.map(|page| self.url(page)),
            "label": "&laquo; Previous",
            "page": previous,
            "active": false,
        })];
        for item in self.numbered() {
            links.push(match item {
                Some(page) => json!({
                    "url": self.url(page),
                    "label": page.to_string(),
                    "page": page,
                    "active": page == self.current_page,
                }),
                None => json!({ "url": null, "label": "...", "page": null, "active": false }),
            });
        }
        links.push(json!({
            "url": next.map(|page| self.url(page)),
            "label": "Next &raquo;",
            "page": next,
            "active": false,
        }));
        json!({
            "current_page": self.current_page,
            "data": data,
            "first_page_url": self.url(1),
            "from": from,
            "last_page": last,
            "last_page_url": self.url(last),
            "links": links,
            "next_page_url": next.map(|page| self.url(page)),
            "path": self.path,
            "per_page": self.per_page,
            "prev_page_url": previous.map(|page| self.url(page)),
            "to": to,
            "total": self.total,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(query: &[(&str, &str)]) -> Vec<(String, String)> {
        query
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn keeps_the_query_and_puts_the_page_last() {
        let query = pairs(&[("page", "2"), ("view", "corporations"), ("search", " a b ")]);
        let paginator = Paginator {
            path: "https://x/admin/pilots".to_owned(),
            query: &query,
            per_page: 20,
            current_page: current_page(&query),
            total: 25,
        };
        assert_eq!(
            paginator.url(1),
            "https://x/admin/pilots?view=corporations&search=%20a%20b%20&page=1"
        );
        let value = paginator.json(vec![json!(1)]);
        assert_eq!(value["from"], 21);
        assert_eq!(value["to"], 21);
        assert_eq!(value["last_page"], 2);
        assert_eq!(value["next_page_url"], Value::Null);
    }

    #[test]
    fn windows_many_pages() {
        let query = pairs(&[("page", "10")]);
        let paginator = Paginator {
            path: "p".to_owned(),
            query: &query,
            per_page: 1,
            current_page: 10,
            total: 30,
        };
        let labels: Vec<String> = paginator.json(vec![])["links"]
            .as_array()
            .expect("links")
            .iter()
            .map(|link| link["label"].as_str().expect("a label").to_owned())
            .collect();
        assert_eq!(
            labels,
            [
                "&laquo; Previous",
                "1",
                "2",
                "...",
                "7",
                "8",
                "9",
                "10",
                "11",
                "12",
                "13",
                "...",
                "29",
                "30",
                "Next &raquo;"
            ]
        );
    }

    #[test]
    fn reads_the_current_page() {
        assert_eq!(current_page(&pairs(&[("page", "3")])), 3);
        assert_eq!(current_page(&pairs(&[("page", "0")])), 1);
        assert_eq!(current_page(&pairs(&[("page", "x")])), 1);
        assert_eq!(current_page(&[]), 1);
    }
}
