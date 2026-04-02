use openssl::symm::{decrypt_aead, encrypt_aead, Cipher};
use rand::Rng;

const AES_256_GCM_KEY_LEN: usize = 32;
const AES_256_GCM_IV_LEN: usize = 12;
const AES_256_GCM_TAG_LEN: usize = 16;

/// Encrypt plaintext with AES-256-GCM.
/// Returns (ciphertext_hex, iv_hex, tag_hex).
pub fn encrypt(plaintext: &str, key_hex: &str) -> Result<(String, String, String), String> {
    let key = hex::decode(key_hex).map_err(|e| format!("invalid key hex: {}", e))?;
    if key.len() != AES_256_GCM_KEY_LEN {
        return Err(format!(
            "key must be {} bytes, got {}",
            AES_256_GCM_KEY_LEN,
            key.len()
        ));
    }

    let iv: [u8; AES_256_GCM_IV_LEN] = rand::thread_rng().gen();
    let mut tag = [0u8; AES_256_GCM_TAG_LEN];

    let cipher = Cipher::aes_256_gcm();
    let ciphertext = encrypt_aead(cipher, &key, Some(&iv), &[], plaintext.as_bytes(), &mut tag)
        .map_err(|e| format!("encrypt failed: {}", e))?;

    Ok((hex::encode(ciphertext), hex::encode(iv), hex::encode(tag)))
}

/// Decrypt AES-256-GCM ciphertext.
/// Takes (ciphertext_hex, iv_hex, tag_hex, key_hex).
pub fn decrypt(
    ciphertext_hex: &str,
    iv_hex: &str,
    tag_hex: &str,
    key_hex: &str,
) -> Result<String, String> {
    let key = hex::decode(key_hex).map_err(|e| format!("invalid key hex: {}", e))?;
    let ciphertext =
        hex::decode(ciphertext_hex).map_err(|e| format!("invalid ciphertext hex: {}", e))?;
    let iv = hex::decode(iv_hex).map_err(|e| format!("invalid iv hex: {}", e))?;
    let tag = hex::decode(tag_hex).map_err(|e| format!("invalid tag hex: {}", e))?;

    if key.len() != AES_256_GCM_KEY_LEN {
        return Err(format!("key must be {} bytes", AES_256_GCM_KEY_LEN));
    }

    let cipher = Cipher::aes_256_gcm();
    let plaintext = decrypt_aead(cipher, &key, Some(&iv), &[], &ciphertext, &tag)
        .map_err(|e| format!("decrypt failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("utf8 decode failed: {}", e))
}

/// Ensure an encryption key exists in the database, creating one if needed.
/// Returns the current key hex string.
pub fn ensure_encryption_key(db: &crate::Database) -> Result<String, String> {
    // Try to get existing key
    if let Ok(Some(key)) = db.get_current_encryption_key() {
        return Ok(key.key_value);
    }

    // Generate new key
    let key_bytes: [u8; AES_256_GCM_KEY_LEN] = rand::thread_rng().gen();
    let key_hex = hex::encode(key_bytes);
    let key_id = format!("key-{}", hex::encode::<[u8; 4]>(rand::thread_rng().gen()));

    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let next = now + 30 * 24 * 60 * 60 * 1000; // 30 days

    let enc_key = crate::EncryptionKey {
        id: None,
        key_id,
        key_value: key_hex.clone(),
        is_current: true,
        created_at: None,
        last_rotation: now,
        next_rotation: next,
    };

    db.save_encryption_key(&enc_key)
        .map_err(|e| format!("save encryption key failed: {}", e))?;

    Ok(key_hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key_hex = hex::encode([42u8; 32]);
        let plaintext = "hello world password test";
        let (ct, iv, tag) = encrypt(plaintext, &key_hex).unwrap();
        let decrypted = decrypt(&ct, &iv, &tag, &key_hex).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key_hex = hex::encode([42u8; 32]);
        let wrong_key = hex::encode([99u8; 32]);
        let (ct, iv, tag) = encrypt("secret", &key_hex).unwrap();
        assert!(decrypt(&ct, &iv, &tag, &wrong_key).is_err());
    }
}
