use serde::{Deserialize, Deserializer, Serialize, Serializer};
use specta::Type;
use std::marker::PhantomData;

use super::state::{CipherState, Decrypted, Encrypted};

/// A field that can be in either encrypted or decrypted state
///
/// This type provides type-safe encapsulation of cipher fields,
/// ensuring that encrypted and decrypted data are tracked at compile time.
#[derive(Debug, Clone, PartialEq, Eq, Type)]
#[specta(inline)]
pub struct EncryptedField<S, T>
where
    S: CipherState,
{
    inner: Option<T>,
    #[serde(skip)]
    _state: PhantomData<S>,
}

/// Type alias for encrypted string fields
pub type EncryptedString = EncryptedField<Encrypted, String>;

/// Type alias for decrypted string fields
pub type DecryptedString = EncryptedField<Decrypted, String>;

impl<S, T> EncryptedField<S, T>
where
    S: CipherState,
{
    /// Creates a new field with the given value
    pub fn new(value: Option<T>) -> Self {
        Self {
            inner: value,
            _state: PhantomData,
        }
    }

    /// Creates a field with no value
    pub fn none() -> Self {
        Self {
            inner: None,
            _state: PhantomData,
        }
    }

    /// Returns a reference to the inner value
    pub fn as_ref(&self) -> Option<&T> {
        self.inner.as_ref()
    }

    /// Returns a mutable reference to the inner value
    pub fn as_mut(&mut self) -> Option<&mut T> {
        self.inner.as_mut()
    }

    /// Maps the inner value using a function
    pub fn map<U, F>(self, f: F) -> EncryptedField<S, U>
    where
        F: FnOnce(T) -> U,
        S: CipherState,
    {
        EncryptedField {
            inner: self.inner.map(f),
            _state: PhantomData,
        }
    }

    /// Converts into the inner value
    pub fn into_inner(self) -> Option<T> {
        self.inner
    }

    /// Checks if the field has a value
    pub fn is_some(&self) -> bool {
        self.inner.is_some()
    }

    /// Checks if the field has no value
    pub fn is_none(&self) -> bool {
        self.inner.is_none()
    }

    /// Checks if the field should be skipped during serialization
    /// This is used for skip_serializing_if attribute
    pub fn should_skip(&self) -> bool {
        self.inner.is_none()
    }

    /// Transposes the field to an option of state
    pub fn transpose(self) -> Option<EncryptedField<S, T>> {
        if self.inner.is_some() {
            Some(self)
        } else {
            None
        }
    }
}

/// Helper function for skip_serializing_if attribute
pub fn should_skip_field<S, T>(field: &EncryptedField<S, T>) -> bool
where
    S: CipherState,
{
    field.is_none()
}

impl<S, T> Default for EncryptedField<S, T>
where
    S: CipherState,
{
    fn default() -> Self {
        Self {
            inner: None,
            _state: PhantomData,
        }
    }
}

/// Transparent serialization - delegates to inner Option<T>
impl<S, T> Serialize for EncryptedField<S, T>
where
    S: CipherState,
    T: Serialize,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        self.inner.serialize(serializer)
    }
}

/// Transparent deserialization - delegates from inner Option<T>
impl<'de, S, T> Deserialize<'de> for EncryptedField<S, T>
where
    S: CipherState,
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = Option::<T>::deserialize(deserializer)?;
        Ok(Self {
            inner,
            _state: PhantomData,
        })
    }
}

impl<S, T> From<Option<T>> for EncryptedField<S, T>
where
    S: CipherState,
{
    fn from(value: Option<T>) -> Self {
        Self::new(value)
    }
}

impl<S, T> From<T> for EncryptedField<S, T>
where
    S: CipherState,
{
    fn from(value: T) -> Self {
        Self::new(Some(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_field_new() {
        let field: EncryptedString = EncryptedField::new(Some("test".to_string()));
        assert_eq!(field.as_ref(), Some(&"test".to_string()));
    }

    #[test]
    fn test_encrypted_field_none() {
        let field: EncryptedString = EncryptedField::none();
        assert!(field.is_none());
        assert!(!field.is_some());
    }

    #[test]
    fn test_encrypted_field_map() {
        let field: EncryptedString = EncryptedField::new(Some("encrypted".to_string()));
        let mapped = field.map(|s| format!("decrypted: {}", s));
        assert_eq!(mapped.as_ref(), Some(&"decrypted: encrypted".to_string()));
    }

    #[test]
    fn test_encrypted_field_into_inner() {
        let field: EncryptedString = EncryptedField::new(Some("test".to_string()));
        assert_eq!(field.into_inner(), Some("test".to_string()));
    }

    #[test]
    fn test_encrypted_field_default() {
        let field: EncryptedString = EncryptedField::default();
        assert!(field.is_none());
    }

    #[test]
    fn test_encrypted_field_from_option() {
        let field: EncryptedString = Option::Some("test".to_string()).into();
        assert_eq!(field.as_ref(), Some(&"test".to_string()));
    }

    #[test]
    fn test_encrypted_field_from_value() {
        let field: EncryptedString = "test".to_string().into();
        assert_eq!(field.as_ref(), Some(&"test".to_string()));
    }

    #[test]
    fn test_encrypted_field_serialize() {
        let field: EncryptedString = EncryptedField::new(Some("test".to_string()));
        let json = serde_json::to_string(&field).unwrap();
        assert_eq!(json, r#""test""#);
    }

    #[test]
    fn test_encrypted_field_deserialize() {
        let field: EncryptedString = serde_json::from_str(r#""test""#).unwrap();
        assert_eq!(field.as_ref(), Some(&"test".to_string()));
    }

    #[test]
    fn test_encrypted_field_serialize_none() {
        let field: EncryptedString = EncryptedField::none();
        let json = serde_json::to_string(&field).unwrap();
        assert_eq!(json, "null");
    }

    #[test]
    fn test_encrypted_field_deserialize_none() {
        let field: EncryptedString = serde_json::from_str("null").unwrap();
        assert!(field.is_none());
    }
}
