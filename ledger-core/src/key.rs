//! The decision key — the stable human name a generated type is built from.
//!
//! A decision's id is `dec:<namespace>/<ulid>`: permanent, but nobody can
//! cite a ULID from code. The key (`MoneyIsDecimal`) is what the analyzers'
//! generator turns into a nested type name, so it is constrained to what a
//! C# identifier can carry and pinned once given: immutable along a
//! decision's own version chain (`L013`) and unique among the live
//! decisions of one namespace (`L014`). Both rules are file-gate classes,
//! because nothing a generated type name depends on may be graph-only
//! (PRD §0 item 2).
//!
//! A malformed key is refused at parse, as a `SCHEMA` fault — the same
//! posture every other typed wire token has.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The longest key the format admits: one uppercase letter plus 63 more.
pub const MAX_LEN: usize = 64;

/// A key matching `^[A-Z][A-Za-z0-9]{0,63}$`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecisionKey(String);

impl DecisionKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for DecisionKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let Some(first) = chars.next() else {
            return Err("a key is empty — expected `^[A-Z][A-Za-z0-9]{0,63}$`".to_string());
        };
        if !first.is_ascii_uppercase() {
            return Err(format!("`{s}` does not start with an uppercase ASCII letter"));
        }
        if let Some(bad) = chars.find(|c| !c.is_ascii_alphanumeric()) {
            return Err(format!("`{s}` carries `{bad}`; a key is ASCII letters and digits only"));
        }
        if s.len() > MAX_LEN {
            return Err(format!("`{s}` is {} characters; a key is at most {MAX_LEN}", s.len()));
        }
        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for DecisionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for DecisionKey {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for DecisionKey {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pascal_case_key_parses() {
        let key: DecisionKey = "MoneyIsDecimal2".parse().expect("key");
        assert_eq!(key.as_str(), "MoneyIsDecimal2");
    }

    #[test]
    fn the_first_character_is_an_uppercase_letter() {
        assert!("moneyIsDecimal".parse::<DecisionKey>().is_err());
        assert!("2Money".parse::<DecisionKey>().is_err());
        assert!("".parse::<DecisionKey>().is_err());
    }

    #[test]
    fn only_ascii_letters_and_digits_follow() {
        for bad in ["Money_Decimal", "Money-Decimal", "Money Decimal", "Mønster"] {
            assert!(bad.parse::<DecisionKey>().is_err(), "{bad} should be refused");
        }
    }

    #[test]
    fn sixty_four_characters_is_the_limit() {
        let at_limit = format!("A{}", "b".repeat(63));
        assert!(at_limit.parse::<DecisionKey>().is_ok());
        let over = format!("A{}", "b".repeat(64));
        let err = over.parse::<DecisionKey>().expect_err("too long");
        assert!(err.contains("at most 64"), "{err}");
    }

    #[test]
    fn a_bad_key_in_a_file_is_a_parse_error() {
        let err = serde_yaml::from_str::<DecisionKey>("lower").expect_err("refused");
        assert!(err.to_string().contains("uppercase"), "{err}");
    }
}
