use md5::{Digest, Md5};

/// Generate a deterministic UUID from a seed string.
///
/// Format: `XX` + first 20 hex chars of `md5(seed)` + `XX`
/// If the UUID already exists in the `existing` set, append a space to the seed and retry.
pub fn generate_uuid(seed: &str, existing: &std::collections::HashSet<String>) -> String {
    let mut current_seed = seed.to_string();
    loop {
        let uuid = make_uuid(&current_seed);
        if !existing.contains(&uuid) {
            return uuid;
        }
        current_seed.push(' ');
    }
}

/// Generate a deterministic dashed UUID (8-4-4-4-12) from a seed string.
pub fn generate_dashed_uuid(seed: &str) -> String {
    let hex = md5_hex(seed);
    format!("{}-{}-{}-{}-{}", &hex[..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..32])
}

fn md5_hex(seed: &str) -> String {
    let result = Md5::digest(seed.as_bytes());
    result.iter().map(|b| format!("{:02X}", b)).collect()
}

fn make_uuid(seed: &str) -> String {
    let hex = md5_hex(seed);
    format!("XX{}XX", &hex[..20])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_uuid_generation() {
        let existing = HashSet::new();
        let uuid = generate_uuid("test-seed", &existing);
        assert_eq!(uuid.len(), 24);
        assert!(uuid.starts_with("XX"));
        assert!(uuid.ends_with("XX"));
    }

    #[test]
    fn test_uuid_deterministic() {
        let existing = HashSet::new();
        let uuid1 = generate_uuid("same-seed", &existing);
        let uuid2 = generate_uuid("same-seed", &existing);
        assert_eq!(uuid1, uuid2);
    }

    #[test]
    fn test_uuid_collision_avoidance() {
        let uuid1 = make_uuid("test");
        let mut existing = HashSet::new();
        existing.insert(uuid1.clone());
        let uuid2 = generate_uuid("test", &existing);
        assert_ne!(uuid1, uuid2);
    }
}
