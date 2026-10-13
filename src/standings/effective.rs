//! The standing that applies to an entity (`EffectiveStandingResolver`):
//! its own contact standing, or the one inherited from a parent
//! corporation or alliance, or 10 for the source itself and its members.

use std::collections::HashMap;

use serde_json::{Value, json};
use sqlx::PgPool;

use crate::format::php_float;
use crate::standings::{Contact, Source, contacts, key};

pub struct Resolver {
    contacts: HashMap<String, Contact>,
    source: Option<Source>,
}

/// A standing request's subject and its requester's affiliations.
pub struct RequestChain {
    pub subject_type: String,
    pub subject_id: i64,
    pub requester_corporation_id: Option<i64>,
    pub requester_alliance_id: Option<i64>,
}

impl Resolver {
    pub async fn load(pool: &PgPool) -> sqlx::Result<Self> {
        let contacts = contacts(pool, "c.id")
            .await?
            .into_iter()
            .map(|contact| (contact.key(), contact))
            .collect();
        Ok(Self {
            contacts,
            source: Source::current(pool).await?,
        })
    }

    /// The standing of a request's subject, walking up from the subject
    /// to the requester's corporation and alliance: `{standing, source:
    /// direct|corporation|alliance, via_type, via_id, via_name}` or null.
    pub fn resolve(&self, request: &RequestChain) -> Value {
        let mut chain = vec![(
            "direct",
            request.subject_type.as_str(),
            Some(request.subject_id),
        )];
        if request.subject_type == "character" && request.requester_corporation_id.is_some() {
            chain.push((
                "corporation",
                "corporation",
                request.requester_corporation_id,
            ));
        }
        if request.subject_type != "alliance" && request.requester_alliance_id.is_some() {
            chain.push(("alliance", "alliance", request.requester_alliance_id));
        }
        for (source, kind, id) in chain {
            let Some(id) = id else { continue };
            if let Some(contact) = self.contacts.get(&key(kind, id)) {
                return json!({
                    "standing": php_float(contact.standing),
                    "source": source,
                    "via_type": kind,
                    "via_id": id,
                    "via_name": contact.name,
                });
            }
        }
        Value::Null
    }

    /// A character's standing: 10 when the source covers it, else the
    /// first of its own, its corporation's and its alliance's.
    pub fn for_character(
        &self,
        id: i64,
        corporation_id: Option<i64>,
        alliance_id: Option<i64>,
    ) -> Value {
        if self
            .source
            .as_ref()
            .is_some_and(|source| source.covers(corporation_id, alliance_id))
        {
            return json!({ "value": php_float(10.0), "source": "source" });
        }
        self.first(&[
            ("character", Some(id)),
            ("corporation", corporation_id),
            ("alliance", alliance_id),
        ])
    }

    /// A corporation's (with its alliance) or an alliance's standing.
    pub fn for_entity(&self, kind: &str, id: i64, alliance_id: Option<i64>) -> Value {
        let covered = self.source.as_ref().is_some_and(|source| {
            if kind == "corporation" {
                (source.kind == "corporation" && source.entity_id == id)
                    || (source.kind == "alliance" && Some(source.entity_id) == alliance_id)
            } else {
                source.kind == "alliance" && source.entity_id == id
            }
        });
        if covered {
            return json!({ "value": php_float(10.0), "source": "source" });
        }
        if kind == "corporation" {
            self.first(&[("corporation", Some(id)), ("alliance", alliance_id)])
        } else {
            self.first(&[("alliance", Some(id))])
        }
    }

    fn first(&self, chain: &[(&str, Option<i64>)]) -> Value {
        for (kind, id) in chain {
            let Some(id) = id else { continue };
            if let Some(contact) = self.contacts.get(&key(kind, *id)) {
                return json!({ "value": php_float(contact.standing), "source": kind });
            }
        }
        Value::Null
    }
}
