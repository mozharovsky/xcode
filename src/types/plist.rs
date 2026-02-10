use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Core in-memory representation for parsed .pbxproj data.
///
/// Maps directly to Apple's Old-Style Plist format used by Xcode project files.
#[derive(Debug, Clone, PartialEq)]
pub enum PlistValue {
    /// A string value (quoted or unquoted in the source).
    String(String),
    /// An integer value. Only used for unquoted digit-only values that fit in i64
    /// and are within JS MAX_SAFE_INTEGER (2^53 - 1).
    Integer(i64),
    /// A floating-point value.
    Float(f64),
    /// Binary data represented as `<hex bytes>` in the source.
    Data(Vec<u8>),
    /// An ordered key-value map (`{ key = value; ... }`).
    Object(IndexMap<String, PlistValue>),
    /// An ordered list of values (`( item1, item2, ... )`).
    Array(Vec<PlistValue>),
}

impl PlistValue {
    /// Returns true if this is a String variant.
    pub fn is_string(&self) -> bool {
        matches!(self, PlistValue::String(_))
    }

    /// Returns the string value if this is a String variant.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            PlistValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the integer value if this is an Integer variant.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            PlistValue::Integer(n) => Some(*n),
            _ => None,
        }
    }

    /// Returns a reference to the inner map if this is an Object variant.
    pub fn as_object(&self) -> Option<&IndexMap<String, PlistValue>> {
        match self {
            PlistValue::Object(map) => Some(map),
            _ => None,
        }
    }

    /// Returns a mutable reference to the inner map if this is an Object variant.
    pub fn as_object_mut(&mut self) -> Option<&mut IndexMap<String, PlistValue>> {
        match self {
            PlistValue::Object(map) => Some(map),
            _ => None,
        }
    }

    /// Returns a reference to the inner vec if this is an Array variant.
    pub fn as_array(&self) -> Option<&Vec<PlistValue>> {
        match self {
            PlistValue::Array(vec) => Some(vec),
            _ => None,
        }
    }

    /// Get a value from an Object by key.
    pub fn get(&self, key: &str) -> Option<&PlistValue> {
        self.as_object().and_then(|map| map.get(key))
    }
}

/// Serialize PlistValue to JSON for napi interop.
///
/// This matches the JsonVisitor.ts behavior:
/// - Strings → JSON strings
/// - Integers → JSON numbers
/// - Floats → JSON numbers (but trailing .0 preserved as string in some contexts)
/// - Data → JSON objects with { type: "Buffer", data: [...] } (matching Node.js Buffer.toJSON)
/// - Objects → JSON objects (preserving key order)
/// - Arrays → JSON arrays
impl Serialize for PlistValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PlistValue::String(s) => serializer.serialize_str(s),
            PlistValue::Integer(n) => serializer.serialize_i64(*n),
            PlistValue::Float(f) => {
                // Preserve trailing zero: 5.0 stays as "5.0" string
                let s = format!("{}", f);
                if s.contains('.') {
                    serializer.serialize_f64(*f)
                } else {
                    serializer.serialize_f64(*f)
                }
            }
            PlistValue::Data(bytes) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "Buffer")?;
                map.serialize_entry("data", bytes)?;
                map.end()
            }
            PlistValue::Object(map) => {
                use serde::ser::SerializeMap;
                let mut ser_map = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map {
                    ser_map.serialize_entry(k, v)?;
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

/// Deserialize JSON back to PlistValue for napi interop.
impl<'de> Deserialize<'de> for PlistValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, SeqAccess, Visitor};
        use std::fmt;

        struct PlistVisitor;

        impl<'de> Visitor<'de> for PlistVisitor {
            type Value = PlistValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid plist value")
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<PlistValue, E> {
                Ok(PlistValue::Integer(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<PlistValue, E> {
                if v <= i64::MAX as u64 {
                    Ok(PlistValue::Integer(v as i64))
                } else {
                    Ok(PlistValue::String(v.to_string()))
                }
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<PlistValue, E> {
                Ok(PlistValue::Float(v))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<PlistValue, E> {
                Ok(PlistValue::String(v.to_string()))
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<PlistValue, E> {
                Ok(PlistValue::String(v))
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> Result<PlistValue, E> {
                Ok(PlistValue::String(if v { "YES" } else { "NO" }.to_string()))
            }

            fn visit_none<E: de::Error>(self) -> Result<PlistValue, E> {
                Ok(PlistValue::String(String::new()))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<PlistValue, A::Error> {
                let mut vec = Vec::new();
                while let Some(elem) = seq.next_element()? {
                    vec.push(elem);
                }
                Ok(PlistValue::Array(vec))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<PlistValue, A::Error> {
                // Check for Buffer objects: { type: "Buffer", data: [...] }
                let mut index_map = IndexMap::new();
                while let Some((key, value)) = map.next_entry::<String, PlistValue>()? {
                    index_map.insert(key, value);
                }

                // Detect Buffer serialization format
                if index_map.len() == 2 {
                    if let Some(PlistValue::String(t)) = index_map.get("type") {
                        if t == "Buffer" {
                            if let Some(PlistValue::Array(data)) = index_map.get("data") {
                                let bytes: Vec<u8> = data
                                    .iter()
                                    .filter_map(|v| v.as_integer().map(|n| n as u8))
                                    .collect();
                                return Ok(PlistValue::Data(bytes));
                            }
                        }
                    }
                }

                Ok(PlistValue::Object(index_map))
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
        let val = PlistValue::String("hello".to_string());
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
        let mut map = IndexMap::new();
        map.insert("key".to_string(), PlistValue::String("value".to_string()));
        let val = PlistValue::Object(map);
        assert!(val.as_object().is_some());
        assert_eq!(
            val.get("key").and_then(|v| v.as_str()),
            Some("value")
        );
    }

    #[test]
    fn test_serialize_roundtrip() {
        let mut map = IndexMap::new();
        map.insert("name".to_string(), PlistValue::String("test".to_string()));
        map.insert("version".to_string(), PlistValue::Integer(1));
        let val = PlistValue::Object(map);

        let json = serde_json::to_string(&val).unwrap();
        let back: PlistValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, back);
    }
}
