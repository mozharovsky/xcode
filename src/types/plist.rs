use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Hash-indexed map used by PbxObject/XcodeProject for O(1) lookups.
pub type PlistMap<'a> = indexmap::IndexMap<Cow<'a, str>, PlistValue<'a>, ahash::RandomState>;

/// Ordered key-value pairs — the Object storage in PlistValue.
/// Uses Vec for zero-overhead construction during parsing.
pub type PlistObject<'a> = Vec<(Cow<'a, str>, PlistValue<'a>)>;

/// Core in-memory representation for parsed .pbxproj data.
///
/// The lifetime `'a` enables zero-copy parsing: both keys and values can borrow
/// directly from the input text. Use `PlistValue<'static>` (via `into_owned()`)
/// when long-lived ownership is needed (e.g. in `XcodeProject`).
#[derive(Debug, Clone, PartialEq)]
pub enum PlistValue<'a> {
    /// A string value (quoted or unquoted in the source).
    String(Cow<'a, str>),
    /// An integer value.
    Integer(i64),
    /// A floating-point value.
    Float(f64),
    /// Binary data represented as `<hex bytes>` in the source.
    Data(Vec<u8>),
    /// An ordered key-value list (`{ key = value; ... }`).
    Object(PlistObject<'a>),
    /// An ordered list of values (`( item1, item2, ... )`).
    Array(Vec<PlistValue<'a>>),
}

impl<'a> PlistValue<'a> {
    /// Convert all borrowed strings to owned, producing a `PlistValue<'static>`.
    pub fn into_owned(self) -> PlistValue<'static> {
        match self {
            PlistValue::String(s) => PlistValue::String(Cow::Owned(s.into_owned())),
            PlistValue::Integer(n) => PlistValue::Integer(n),
            PlistValue::Float(f) => PlistValue::Float(f),
            PlistValue::Data(d) => PlistValue::Data(d),
            PlistValue::Object(pairs) => {
                let owned: PlistObject<'static> =
                    pairs.into_iter().map(|(k, v)| (Cow::Owned(k.into_owned()), v.into_owned())).collect();
                PlistValue::Object(owned)
            }
            PlistValue::Array(vec) => PlistValue::Array(vec.into_iter().map(|v| v.into_owned()).collect()),
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, PlistValue::String(_))
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            PlistValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            PlistValue::Integer(n) => Some(*n),
            _ => None,
        }
    }

    /// Returns a reference to the inner pairs if this is an Object variant.
    pub fn as_object(&self) -> Option<&PlistObject<'a>> {
        match self {
            PlistValue::Object(pairs) => Some(pairs),
            _ => None,
        }
    }

    /// Returns a mutable reference to the inner pairs if this is an Object variant.
    pub fn as_object_mut(&mut self) -> Option<&mut PlistObject<'a>> {
        match self {
            PlistValue::Object(pairs) => Some(pairs),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<PlistValue<'a>>> {
        match self {
            PlistValue::Array(vec) => Some(vec),
            _ => None,
        }
    }

    /// Get a value from an Object by key (linear scan — fast for typical <20-key objects).
    pub fn get(&self, key: &str) -> Option<&PlistValue<'a>> {
        self.as_object().and_then(|pairs| pairs.iter().find(|(k, _)| k.as_ref() == key).map(|(_, v)| v))
    }
}

/// Serialize PlistValue to JSON.
impl<'a> Serialize for PlistValue<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PlistValue::String(s) => serializer.serialize_str(s),
            PlistValue::Integer(n) => serializer.serialize_i64(*n),
            PlistValue::Float(f) => serializer.serialize_f64(*f),
            PlistValue::Data(bytes) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "Buffer")?;
                map.serialize_entry("data", bytes)?;
                map.end()
            }
            PlistValue::Object(pairs) => {
                use serde::ser::SerializeMap;
                let mut ser_map = serializer.serialize_map(Some(pairs.len()))?;
                for (k, v) in pairs {
                    ser_map.serialize_entry(k.as_ref(), v)?;
                }
                ser_map.end()
            }
            PlistValue::Array(vec) => {
                use serde::ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(Some(vec.len()))?;
                for item in vec {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
        }
    }
}

/// Deserialize JSON back to PlistValue (always produces owned / 'static values).
impl<'de> Deserialize<'de> for PlistValue<'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, SeqAccess, Visitor};
        use std::fmt;

        struct PlistVisitor;

        impl<'de> Visitor<'de> for PlistVisitor {
            type Value = PlistValue<'static>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid plist value")
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<PlistValue<'static>, E> {
                Ok(PlistValue::Integer(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<PlistValue<'static>, E> {
                if v <= i64::MAX as u64 {
                    Ok(PlistValue::Integer(v as i64))
                } else {
                    Ok(PlistValue::String(Cow::Owned(v.to_string())))
                }
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<PlistValue<'static>, E> {
                Ok(PlistValue::Float(v))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<PlistValue<'static>, E> {
                Ok(PlistValue::String(Cow::Owned(v.to_string())))
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<PlistValue<'static>, E> {
                Ok(PlistValue::String(Cow::Owned(v)))
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> Result<PlistValue<'static>, E> {
                Ok(PlistValue::String(Cow::Owned(if v { "YES" } else { "NO" }.to_string())))
            }

            fn visit_none<E: de::Error>(self) -> Result<PlistValue<'static>, E> {
                Ok(PlistValue::String(Cow::Owned(String::new())))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<PlistValue<'static>, A::Error> {
                let mut vec = Vec::new();
                while let Some(elem) = seq.next_element()? {
                    vec.push(elem);
                }
                Ok(PlistValue::Array(vec))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<PlistValue<'static>, A::Error> {
                let mut pairs = PlistObject::new();
                while let Some((key, value)) = map.next_entry::<String, PlistValue<'static>>()? {
                    pairs.push((Cow::Owned(key), value));
                }

                if pairs.len() == 2 {
                    let has_buffer = pairs.iter().find(|(k, _)| k.as_ref() == "type").and_then(|(_, v)| v.as_str())
                        == Some("Buffer");
                    if has_buffer {
                        if let Some((_, PlistValue::Array(data))) = pairs.iter().find(|(k, _)| k.as_ref() == "data") {
                            let bytes: Vec<u8> = data.iter().filter_map(|v| v.as_integer().map(|n| n as u8)).collect();
                            return Ok(PlistValue::Data(bytes));
                        }
                    }
                }

                Ok(PlistValue::Object(pairs))
            }
        }

        deserializer.deserialize_any(PlistVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plist_value_string() {
        let val = PlistValue::String(Cow::Owned("hello".to_string()));
        assert_eq!(val.as_str(), Some("hello"));
        assert!(val.is_string());
    }

    #[test]
    fn test_plist_value_integer() {
        let val = PlistValue::Integer(42);
        assert_eq!(val.as_integer(), Some(42));
    }

    #[test]
    fn test_plist_value_object() {
        let pairs: PlistObject = vec![(Cow::Borrowed("key"), PlistValue::String(Cow::Owned("value".to_string())))];
        let val = PlistValue::Object(pairs);
        assert!(val.as_object().is_some());
        assert_eq!(val.get("key").and_then(|v| v.as_str()), Some("value"));
    }

    #[test]
    fn test_serialize_roundtrip() {
        let pairs: PlistObject<'static> = vec![
            (Cow::Owned("name".to_string()), PlistValue::String(Cow::Owned("test".to_string()))),
            (Cow::Owned("version".to_string()), PlistValue::Integer(1)),
        ];
        let val: PlistValue<'static> = PlistValue::Object(pairs);

        let json = serde_json::to_string(&val).unwrap();
        let back: PlistValue<'static> = serde_json::from_str(&json).unwrap();
        assert_eq!(val, back);
    }
}
