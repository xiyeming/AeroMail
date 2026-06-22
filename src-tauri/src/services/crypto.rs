use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};

use crate::error::AeroError;

const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const VERSION: u8 = 1;
const KEYRING_SERVICE: &str = "AeroMail";
const KEYRING_USERNAME: &str = "master-key";

/// Returns the path to the fallback master key file used when the OS keyring
/// is unavailable.
fn fallback_key_path() -> Option<std::path::PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join("AeroMail").join("master.key"))
}

fn decode_hex_key(hex_key: &str) -> Result<[u8; KEY_LEN], AeroError> {
    let mut key = [0u8; KEY_LEN];
    hex::decode_to_slice(hex_key.trim(), &mut key)
        .map_err(|e| AeroError::Internal(format!("failed to decode master key: {e}")))?;
    Ok(key)
}

/// Returns the 32-byte AES-256 master key, creating and persisting it on first
/// use. The key is stored in the OS keyring when available, and a fallback file
/// in the local data directory is always written so the key survives even when
/// the keyring backend is unavailable.
fn get_or_create_master_key() -> Result<[u8; KEY_LEN], AeroError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
        .map_err(|e| AeroError::Internal(format!("failed to access keyring entry: {e}")))?;

    // Prefer the OS keyring if it has a key.
    match entry.get_password() {
        Ok(hex_key) if !hex_key.trim().is_empty() => return decode_hex_key(&hex_key),
        Ok(_) | Err(keyring::Error::NoEntry) => {}
        Err(e) => {
            return Err(AeroError::Internal(format!(
                "failed to retrieve master key: {e}"
            )));
        }
    }

    // Fall back to the local file if the keyring has no entry.
    if let Some(path) = fallback_key_path() {
        if path.exists() {
            let hex_key = std::fs::read_to_string(&path).map_err(|e| {
                AeroError::Internal(format!("failed to read fallback master key: {e}"))
            })?;
            return decode_hex_key(&hex_key);
        }
    }

    // Generate a new key and persist it to both the keyring and the fallback
    // file so that at least one copy is available on subsequent calls.
    let key: [u8; KEY_LEN] = rand::random();
    let hex_key = hex::encode(key);

    let _ = entry.set_password(&hex_key);

    if let Some(path) = fallback_key_path() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AeroError::Internal(format!("failed to create fallback key directory: {e}"))
            })?;
        }
        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)
                .map_err(|e| {
                    AeroError::Internal(format!("failed to create fallback master key file: {e}"))
                })?;
            file.write_all(hex_key.as_bytes()).map_err(|e| {
                AeroError::Internal(format!("failed to write fallback master key: {e}"))
            })?;
        }
        #[cfg(not(unix))]
        {
            std::fs::write(&path, hex_key.as_bytes()).map_err(|e| {
                AeroError::Internal(format!("failed to write fallback master key: {e}"))
            })?;
        }
    }

    Ok(key)
}

/// Encrypts a password using AES-256-GCM with the master key.
///
/// The returned bytes have the layout `[version, nonce, ciphertext+tag]`.
///
/// # Errors
///
/// Returns an error if the keyring is unavailable or encryption fails.
pub fn encrypt_password(plaintext: &[u8]) -> Result<Vec<u8>, AeroError> {
    if plaintext.is_empty() {
        return Ok(Vec::new());
    }

    let key = get_or_create_master_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AeroError::Internal(format!("invalid cipher key: {e}")))?;
    let nonce_bytes: [u8; NONCE_LEN] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| AeroError::Internal(format!("encryption failed: {e}")))?;

    let mut out = Vec::with_capacity(1 + NONCE_LEN + ciphertext.len());
    out.push(VERSION);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypts a password previously encrypted by [`encrypt_password`].
///
/// Passwords stored without the version byte prefix are treated as legacy
/// plaintext and returned as-is.
///
/// # Errors
///
/// Returns an error if the keyring is unavailable or decryption fails.
pub fn decrypt_password(encrypted: &[u8]) -> Result<Vec<u8>, AeroError> {
    if encrypted.is_empty() {
        return Ok(Vec::new());
    }

    if encrypted[0] != VERSION {
        // Legacy plaintext password.
        return Ok(encrypted.to_vec());
    }

    if encrypted.len() <= 1 + NONCE_LEN {
        return Err(AeroError::Internal(
            "encrypted password is too short".to_string(),
        ));
    }

    let key = get_or_create_master_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AeroError::Internal(format!("invalid cipher key: {e}")))?;
    let nonce = Nonce::from_slice(&encrypted[1..=NONCE_LEN]);
    let plaintext = cipher
        .decrypt(nonce, &encrypted[NONCE_LEN + 1..])
        .map_err(|e| AeroError::Internal(format!("decryption failed: {e}")))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, PoisonError};

    use super::*;

    static KEYRING_LOCK: Mutex<()> = Mutex::new(());

    fn cleanup_test_key() {
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME).ok();
        let _ = entry.as_ref().and_then(|e| e.delete_credential().ok());
        if let Some(path) = fallback_key_path() {
            let _ = std::fs::remove_file(&path);
        }
    }

    fn acquire_lock() -> std::sync::MutexGuard<'static, ()> {
        KEYRING_LOCK.lock().unwrap_or_else(PoisonError::into_inner)
    }

    #[test]
    fn roundtrip_password() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = acquire_lock();
        cleanup_test_key();
        let plain = b"my-secret-password";
        let encrypted = encrypt_password(plain)?;
        assert_ne!(encrypted, plain.to_vec());
        let decrypted = decrypt_password(&encrypted)?;
        assert_eq!(decrypted, plain.to_vec());
        Ok(())
    }

    #[test]
    fn legacy_plaintext_passthrough() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = acquire_lock();
        cleanup_test_key();
        let plain = b"legacy-plaintext";
        let decrypted = decrypt_password(plain)?;
        assert_eq!(decrypted, plain.to_vec());
        Ok(())
    }

    #[test]
    fn key_consistency() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = acquire_lock();
        cleanup_test_key();
        let k1 = get_or_create_master_key()?;
        let k2 = get_or_create_master_key()?;
        assert_eq!(k1, k2);
        Ok(())
    }
}
