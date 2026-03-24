use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::support::error::AppError;

/// Trait for types that can be decrypted using a vault user key.
///
/// This trait abstracts the decryption logic to eliminate repetitive
/// boilerplate code throughout the codebase, following DRY and SRP principles.
pub trait Decryptable {
    /// The output type after decryption.
    type Output;

    /// Decrypts this value using the provided user key.
    ///
    /// # Arguments
    /// * `key` - The vault user key material for decryption
    /// * `path` - The field path for error context (e.g., "cipher.name")
    ///
    /// # Returns
    /// The decrypted output or an error
    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError>;
}

// Implement Decryptable for Option<String> - the most common case
impl Decryptable for Option<String> {
    type Output = Option<String>;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        crate::application::vault_crypto::decrypt_optional_field(self, key, path)
    }
}

// Implement Decryptable for Vec<T> where T is Decryptable
impl<T: Decryptable> Decryptable for Vec<T> {
    type Output = Vec<T::Output>;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        self.into_iter()
            .enumerate()
            .map(|(index, item)| {
                let item_path = format!("{path}[{index}]");
                item.decrypt(key, &item_path)
            })
            .collect()
    }
}

// Implement Decryptable for Option<T> where T is Decryptable
impl<T: Decryptable> Decryptable for Option<T> {
    type Output = Option<T::Output>;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        self.map(|item| item.decrypt(key, path)).transpose()
    }
}
