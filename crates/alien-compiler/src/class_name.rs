//! Stable, project-scoped class prefixes for transformed modules.

use sha2::{Digest, Sha256};

pub(crate) fn class_prefix(module_id: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
    let digest = Sha256::digest(module_id.as_bytes());
    let mut prefix = String::with_capacity(25);
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
    prefix.push('_');
    prefix
}

#[cfg(test)]
mod tests {
    use super::class_prefix;

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
        assert_eq!(prefix.len(), 25);
        assert!(
            prefix
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        );
    }
}
