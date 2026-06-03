use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, Error as AeadError, KeyInit},
};
use rand::RngCore;
use sha2::{Digest, Sha256};

pub struct SecurityLayer {
    key: Vec<u8>,
}

impl SecurityLayer {
    pub fn new(secret: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());

        Self {
            key: hasher.finalize().to_vec(),
        }
    }

    // Return Result instead of panicking
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, AeadError> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));

        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);

        let nonce_obj = Nonce::from_slice(&nonce);

        // Handle error gracefully
        let mut ciphertext = cipher.encrypt(nonce_obj, data)?;

        let mut out = nonce.to_vec();
        out.append(&mut ciphertext);

        Ok(out)
    }

    // Return Result instead of panicking
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, AeadError> {
        if data.len() < 12 {
            // Basic check to prevent out-of-bounds before slicing
            return Err(AeadError::from(aes_gcm::Error));
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let (nonce, ciphertext) = data.split_at(12);
        let nonce_obj = Nonce::from_slice(nonce);

        // Propagate the error up to the caller
        cipher.decrypt(nonce_obj, ciphertext)
    }
}
