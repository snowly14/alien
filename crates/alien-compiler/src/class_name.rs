//! Stable, project-scoped class prefixes for transformed modules.

use sha2::{Digest, Sha256};

pub(crate) fn class_prefix(module_id: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
    let digest = Sha256::digest(module_id.as_bytes());
    let mut prefix = String::with_capacity(24);
    prefix.push_str("a_");
    // Encode all 128 bits with a CSS-safe alphabet instead of hexadecimal.
    // The leading `a_` makes every result a valid identifier.
    let mut bits = 0u32;
    let mut available = 0;
    for &byte in &digest[..16] {
        bits = (bits << 8) | u32::from(byte);
        available += 8;
        while available >= 6 {
            available -= 6;
            prefix.push(char::from(ALPHABET[((bits >> available) & 63) as usize]));
        }
    }
    if available > 0 {
        prefix.push(char::from(
            ALPHABET[((bits << (6 - available)) & 63) as usize],
        ));
    }
    prefix
}

/// A compact project namespace with approximately 23.7 bits of identity.
/// Its first character is always a CSS identifier letter; the remaining three
/// characters use the full CSS-safe base-64 alphabet.
pub(crate) fn compact_project_prefix(identity: &str) -> String {
    const FIRST: &[u8; 52] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const REST: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
    let digest = Sha256::digest(identity.as_bytes());
    let mut value =
        (u32::from(digest[0]) << 16) | (u32::from(digest[1]) << 8) | u32::from(digest[2]);
    let first = FIRST[(value % 52) as usize];
    value /= 52;
    let mut result = String::with_capacity(4);
    result.push(char::from(first));
    for _ in 0..3 {
        result.push(char::from(REST[(value % 64) as usize]));
        value /= 64;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{class_prefix, compact_project_prefix};

    #[test]
    fn separates_the_previously_colliding_modules() {
        assert_ne!(
            class_prefix("src/components/item-51870.tsx"),
            class_prefix("src/components/item-53940.tsx")
        );
    }

    #[test]
    fn is_stable_and_css_safe() {
        let prefix = class_prefix("src/été.tsx");
        assert_eq!(prefix, class_prefix("src/été.tsx"));
        assert!(prefix.starts_with("a_"));
        assert_eq!(prefix.len(), 24);
        assert!(
            prefix
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        );
    }

    #[test]
    fn compact_project_prefix_is_stable_css_safe_and_project_specific() {
        let first = compact_project_prefix("project/a.ts");
        assert_eq!(first, compact_project_prefix("project/a.ts"));
        assert_ne!(first, compact_project_prefix("another/a.ts"));
        assert_eq!(first.len(), 4);
        assert!(first.as_bytes()[0].is_ascii_alphabetic());
        assert!(
            first
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        );
    }
}

/// Bijective base-52 indices keep generated suffixes short and unambiguous.
pub(crate) fn compact_index(mut index: usize) -> String {
    const ALPHABET: &[u8; 52] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut bytes = Vec::new();
    loop {
        bytes.push(ALPHABET[index % 52]);
        if index < 52 {
            break;
        }
        index = index / 52 - 1;
    }
    bytes.reverse();
    bytes.into_iter().map(char::from).collect()
}

#[cfg(test)]
mod compact_tests {
    use super::compact_index;
    use std::collections::BTreeSet;
    #[test]
    fn suffixes_are_unique_across_length_boundaries() {
        assert_eq!(compact_index(0), "a");
        assert_eq!(compact_index(51), "Z");
        assert_eq!(compact_index(52), "aa");
        assert_eq!(compact_index(103), "aZ");
        let names: BTreeSet<_> = (0..10000).map(compact_index).collect();
        assert_eq!(names.len(), 10000);
        assert_ne!(compact_index(usize::MAX), "");
    }
}
