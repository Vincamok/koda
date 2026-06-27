use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use aes_gcm::aead::rand_core::RngCore;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct SecretRef {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub name: String,
    pub encrypted_value: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// AES-256-GCM encryption — key is 32 bytes from env `SECRET_ENCRYPTION_KEY`.
pub struct SecretCrypto {
    cipher: Aes256Gcm,
}

impl SecretCrypto {
    /// `key_hex` must be exactly 64 hex chars (32 bytes).
    pub fn from_hex_key(key_hex: &str) -> anyhow::Result<Self> {
        let key_bytes = hex::decode(key_hex)
            .map_err(|_| anyhow::anyhow!("SECRET_ENCRYPTION_KEY must be hex-encoded"))?;
        if key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("SECRET_ENCRYPTION_KEY must be 32 bytes (64 hex chars)"));
        }
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        Ok(Self { cipher: Aes256Gcm::new(key) })
    }

    pub fn encrypt(&self, plaintext: &str) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("encrypt error: {e}"))?;
        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    pub fn decrypt(&self, ciphertext: &[u8], nonce_bytes: &[u8]) -> anyhow::Result<String> {
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("decrypt error: {e}"))?;
        Ok(String::from_utf8(plaintext)?)
    }
}
