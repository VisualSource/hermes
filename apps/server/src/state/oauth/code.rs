use base64::Engine;
use rand::RngExt;

// expires in 10min
pub fn generate_code() -> String {
    let mut rng = rand::rng();
    let code: String = (0..32)
        .map(|_| {
            let idx = rng.random_range(0..62);
            match idx {
                0..=25 => (b'a' + idx) as char,
                26..=51 => (b'A' + (idx - 26)) as char,
                _ => (b'0' + (idx - 52)) as char,
            }
        })
        .collect();
    code
}

/// RFC 7636 §4.1: `code_verifier` = `[A-Z] / [a-z] / [0-9] / "-" / "." / "_" / "~"`.
/// The same character set applies to `code_challenge` (§4.2) since it's the
/// base64url-no-pad of a SHA-256 digest, a strict subset of unreserved chars.
pub fn is_valid_pkce_charset(s: &str) -> bool {
    s.bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~'))
}

pub fn validate_pkce(challenge: &str, verifier: &str, method: &str) -> bool {
    if verifier.len() < 43 || verifier.len() > 128 {
        return false;
    }
    if !is_valid_pkce_charset(verifier) {
        return false;
    }

    match method {
        "S256" => {
            use sha2::{Digest, Sha256};

            let mut hasher = Sha256::new();
            hasher.update(verifier.as_bytes());
            let result = hasher.finalize();
            let encoded = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(result);

            challenge == encoded
        }
        _ => false,
    }
}
