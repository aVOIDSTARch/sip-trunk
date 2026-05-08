use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ed25519_dalek::{Signature, VerifyingKey};

#[derive(Debug)]
pub struct SignatureError(pub String);

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Verifies a Telnyx webhook signature using Ed25519.
///
/// - `public_key_b64`: base64-encoded 32-byte Ed25519 public key from
///   Mission Control Portal → Account Settings → Keys & Credentials → Public Key
/// - `timestamp`: value of the `telnyx-timestamp` header (Unix seconds)
/// - `body`: raw request body bytes
/// - `signature_b64`: value of the `telnyx-signature-ed25519` header (base64-encoded)
///
/// The signed payload is: `"{timestamp}|{raw_body}"`
pub fn verify_telnyx_signature(
    public_key_b64: &str,
    timestamp: &str,
    body: &[u8],
    signature_b64: &str,
) -> Result<(), SignatureError> {
    let key_bytes = B64
        .decode(public_key_b64)
        .map_err(|_| SignatureError("invalid public key base64 encoding".to_string()))?;

    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| SignatureError("public key must be exactly 32 bytes".to_string()))?;

    let verifying_key = VerifyingKey::from_bytes(&key_array)
        .map_err(|_| SignatureError("invalid Ed25519 public key".to_string()))?;

    let mut payload = format!("{timestamp}|").into_bytes();
    payload.extend_from_slice(body);

    let sig_bytes = B64
        .decode(signature_b64)
        .map_err(|_| SignatureError("invalid signature base64 encoding".to_string()))?;

    let sig_array: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| SignatureError("signature must be exactly 64 bytes".to_string()))?;

    let signature = Signature::from_bytes(&sig_array);

    verifying_key
        .verify_strict(&payload, &signature)
        .map_err(|_| SignatureError("invalid signature".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD as B64, Engine};
    use ed25519_dalek::{Signer, SigningKey};

    // Deterministic test key — fixed 32-byte seed so tests are reproducible.
    fn test_key_pair() -> (SigningKey, String) {
        let seed = [0x42u8; 32];
        let signing_key = SigningKey::from_bytes(&seed);
        let pub_key_b64 = B64.encode(signing_key.verifying_key().as_bytes());
        (signing_key, pub_key_b64)
    }

    fn make_signature(signing_key: &SigningKey, timestamp: &str, body: &[u8]) -> String {
        let mut payload = format!("{timestamp}|").into_bytes();
        payload.extend_from_slice(body);
        B64.encode(signing_key.sign(&payload).to_bytes())
    }

    #[test]
    fn test_valid_signature_passes() {
        let (signing_key, pub_key_b64) = test_key_pair();
        let timestamp = "1700000000";
        let body = b"{\"data\":{\"event_type\":\"call.initiated\"}}";
        let sig = make_signature(&signing_key, timestamp, body);
        assert!(verify_telnyx_signature(&pub_key_b64, timestamp, body, &sig).is_ok());
    }

    #[test]
    fn test_tampered_body_fails() {
        let (signing_key, pub_key_b64) = test_key_pair();
        let timestamp = "1700000000";
        let body = b"{\"data\":{\"event_type\":\"call.initiated\"}}";
        let sig = make_signature(&signing_key, timestamp, body);
        let tampered = b"{\"data\":{\"event_type\":\"call.hangup\"}}";
        assert!(verify_telnyx_signature(&pub_key_b64, timestamp, tampered, &sig).is_err());
    }

    #[test]
    fn test_wrong_key_fails() {
        let (signing_key, _) = test_key_pair();
        let timestamp = "1700000000";
        let body = b"{}";
        let sig = make_signature(&signing_key, timestamp, body);
        // Different key pair for verification
        let other_seed = [0x99u8; 32];
        let other_key = SigningKey::from_bytes(&other_seed);
        let other_pub_b64 = B64.encode(other_key.verifying_key().as_bytes());
        assert!(verify_telnyx_signature(&other_pub_b64, timestamp, body, &sig).is_err());
    }

    #[test]
    fn test_tampered_timestamp_fails() {
        let (signing_key, pub_key_b64) = test_key_pair();
        let body = b"{\"data\":{}}";
        let sig = make_signature(&signing_key, "1700000000", body);
        // Replaying with a different timestamp must fail
        assert!(verify_telnyx_signature(&pub_key_b64, "9999999999", body, &sig).is_err());
    }

    #[test]
    fn test_invalid_public_key_encoding_errors() {
        let result = verify_telnyx_signature("not-valid-base64!!!", "ts", b"body", "sig");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_signature_encoding_errors() {
        let (_, pub_key_b64) = test_key_pair();
        let result = verify_telnyx_signature(&pub_key_b64, "ts", b"body", "not-valid-base64!!!");
        assert!(result.is_err());
    }
}
