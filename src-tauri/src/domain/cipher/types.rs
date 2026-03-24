use serde::{Deserialize, Serialize};
use specta::Type;

use super::field::{should_skip_field, EncryptedField};
use super::state::CipherState;

/// Cipher login data
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(bound = "")]
pub struct CipherLogin<S: CipherState> {
    #[serde(skip_serializing_if = "should_skip_field")]
    pub uri: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub uris: Vec<CipherLoginUri<S>>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub username: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub password: EncryptedField<S, String>,
    /// Plain text field - not encrypted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_revision_date: Option<String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub totp: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autofill_on_page_load: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fido2_credentials: Vec<CipherLoginFido2Credential<S>>,
}

/// Cipher login URI
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(bound = "")]
pub struct CipherLoginUri<S: CipherState> {
    #[serde(skip_serializing_if = "should_skip_field")]
    pub uri: EncryptedField<S, String>,
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub r#match: Option<i32>,
    /// Plain text field - not encrypted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri_checksum: Option<String>,
}

/// Cipher login FIDO2 credential
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(bound = "")]
pub struct CipherLoginFido2Credential<S: CipherState> {
    #[serde(skip_serializing_if = "should_skip_field")]
    pub credential_id: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub key_type: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub key_algorithm: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub key_curve: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub key_value: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub rp_id: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub rp_name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub counter: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub user_handle: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub user_name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub user_display_name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub discoverable: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub creation_date: EncryptedField<S, String>,
}

/// Cipher card data
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(bound = "")]
pub struct CipherCard<S: CipherState> {
    #[serde(skip_serializing_if = "should_skip_field")]
    pub cardholder_name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub brand: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub number: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub exp_month: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub exp_year: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub code: EncryptedField<S, String>,
}

/// Cipher identity data
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(bound = "")]
pub struct CipherIdentity<S: CipherState> {
    #[serde(skip_serializing_if = "should_skip_field")]
    pub title: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub first_name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub middle_name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub last_name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub address1: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub address2: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub address3: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub city: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub state: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub postal_code: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub country: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub company: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub email: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub phone: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub ssn: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub username: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub passport_number: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub license_number: EncryptedField<S, String>,
}

/// Cipher SSH key data
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(bound = "")]
pub struct CipherSshKey<S: CipherState> {
    #[serde(skip_serializing_if = "should_skip_field")]
    pub private_key: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub public_key: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub key_fingerprint: EncryptedField<S, String>,
}

/// Cipher secure note data (no encrypted fields)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CipherSecureNote {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<i32>,
}

/// Cipher attachment data
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(bound = "")]
pub struct CipherAttachment<S: CipherState> {
    pub id: String,
    /// Plain text field - not encrypted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub file_name: EncryptedField<S, String>,
    /// Plain text field - not encrypted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Plain text field - not encrypted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_name: Option<String>,
    /// Plain text field - not encrypted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
}

/// Cipher custom field
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(bound = "")]
pub struct CipherField<S: CipherState> {
    #[serde(skip_serializing_if = "should_skip_field")]
    pub name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub value: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_id: Option<i32>,
}

/// Cipher password history entry
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(bound = "")]
pub struct CipherPasswordHistory<S: CipherState> {
    #[serde(skip_serializing_if = "should_skip_field")]
    pub password: EncryptedField<S, String>,
    /// Plain text field - not encrypted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_date: Option<String>,
}

/// Cipher permissions (no encrypted fields)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CipherPermissions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restore: Option<bool>,
}

/// Cipher data (legacy structure containing all fields)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(bound = "")]
pub struct CipherData<S: CipherState> {
    #[serde(skip_serializing_if = "should_skip_field")]
    pub name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub notes: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fields: Vec<CipherField<S>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub password_history: Vec<CipherPasswordHistory<S>>,
    // Login fields
    #[serde(skip_serializing_if = "should_skip_field")]
    pub uri: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub uris: Vec<CipherLoginUri<S>>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub username: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub password: EncryptedField<S, String>,
    /// Plain text field - not encrypted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_revision_date: Option<String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub totp: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autofill_on_page_load: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fido2_credentials: Vec<CipherLoginFido2Credential<S>>,
    // Card fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<i32>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub cardholder_name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub brand: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub number: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub exp_month: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub exp_year: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub code: EncryptedField<S, String>,
    // Identity fields
    #[serde(skip_serializing_if = "should_skip_field")]
    pub title: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub first_name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub middle_name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub last_name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub address1: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub address2: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub address3: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub city: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub state: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub postal_code: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub country: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub company: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub email: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub phone: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub ssn: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub passport_number: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub license_number: EncryptedField<S, String>,
    // SSH key fields
    #[serde(skip_serializing_if = "should_skip_field")]
    pub private_key: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub public_key: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub key_fingerprint: EncryptedField<S, String>,
}

#[cfg(test)]
mod tests {
    use super::super::state::{Decrypted, Encrypted};
    use super::*;

    #[test]
    fn test_cipher_login_encrypted() {
        let login: CipherLogin<Encrypted> = CipherLogin {
            uri: EncryptedField::new(Some("encrypted_uri".to_string())),
            uris: vec![],
            username: EncryptedField::new(Some("encrypted_username".to_string())),
            password: EncryptedField::new(Some("encrypted_password".to_string())),
            password_revision_date: None,
            totp: EncryptedField::none(),
            autofill_on_page_load: None,
            fido2_credentials: vec![],
        };
        assert!(login.uri.is_some());
    }

    #[test]
    fn test_cipher_login_decrypted() {
        let login: CipherLogin<Decrypted> = CipherLogin {
            uri: EncryptedField::new(Some("decrypted_uri".to_string())),
            uris: vec![],
            username: EncryptedField::new(Some("decrypted_username".to_string())),
            password: EncryptedField::new(Some("decrypted_password".to_string())),
            password_revision_date: None,
            totp: EncryptedField::none(),
            autofill_on_page_load: None,
            fido2_credentials: vec![],
        };
        assert!(login.uri.is_some());
    }

    #[test]
    fn test_cipher_secure_note() {
        let note = CipherSecureNote { r#type: Some(1) };
        assert_eq!(note.r#type, Some(1));
    }

    #[test]
    fn test_cipher_permissions() {
        let permissions = CipherPermissions {
            delete: Some(true),
            restore: Some(false),
        };
        assert_eq!(permissions.delete, Some(true));
    }
}
