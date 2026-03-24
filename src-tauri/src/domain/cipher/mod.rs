mod convert;
mod field;
mod serde_helpers;
mod state;
mod types;

use ::serde::{Deserialize, Deserializer, Serialize, Serializer};
use specta::Type;
use std::marker::PhantomData;

pub use field::{should_skip_field, DecryptedString, EncryptedField, EncryptedString};
pub use state::{CipherState, Decrypted, Encrypted};
pub use types::{
    CipherAttachment, CipherCard, CipherData, CipherField, CipherIdentity, CipherLogin,
    CipherLoginFido2Credential, CipherLoginUri, CipherPasswordHistory, CipherPermissions,
    CipherSecureNote, CipherSshKey,
};

use super::crypto::Decryptable;
use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::support::error::AppError;

/// Main cipher structure with type-state pattern
#[derive(Debug, Clone, Type)]
#[serde(bound = "")]
pub struct Cipher<S: CipherState> {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<i32>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub name: EncryptedField<S, String>,
    #[serde(skip_serializing_if = "should_skip_field")]
    pub notes: EncryptedField<S, String>,
    /// Cipher key - stored as encrypted string, should not be decrypted like regular fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorite: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_password: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_use_totp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reprompt: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<CipherPermissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fields: Vec<CipherField<S>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub password_history: Vec<CipherPasswordHistory<S>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub collection_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<CipherData<S>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<CipherLogin<S>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure_note: Option<CipherSecureNote>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<CipherCard<S>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<CipherIdentity<S>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key: Option<CipherSshKey<S>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attachments: Vec<CipherAttachment<S>>,
    #[serde(skip)]
    _state: PhantomData<S>,
}

impl<S: CipherState> Cipher<S> {
    /// Checks if the cipher has TOTP configured
    pub fn has_totp(&self) -> bool {
        // Check login.totp
        if let Some(ref login) = self.login {
            if login.totp.is_some() {
                return true;
            }
        }

        // Check data.totp
        if let Some(ref data) = self.data {
            if data.totp.is_some() {
                return true;
            }
        }

        false
    }
}

/// Custom serialization for Cipher<Encrypted> - outputs snake_case
impl Serialize for Cipher<Encrypted> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        // First serialize using the standard derive
        let json = serde_json::to_value(self).map_err(::serde::ser::Error::custom)?;
        // Convert all keys to snake_case
        let converted = self::serde_helpers::convert_keys_to_snake_case(json);
        // Serialize the converted value
        converted.serialize(serializer)
    }
}

/// Custom deserialization for Cipher<Encrypted> - accepts snake_case
impl<'de> Deserialize<'de> for Cipher<Encrypted> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // First deserialize as JSON Value
        let json = serde_json::Value::deserialize(deserializer)?;
        // Convert all keys from snake_case to camelCase for struct deserialization
        let converted = self::serde_helpers::convert_keys_to_camel_case(json);
        // Now deserialize the converted value
        serde_json::from_value(converted).map_err(::serde::de::Error::custom)
    }
}

/// Standard serialization for Cipher<Decrypted> - uses camelCase
impl Serialize for Cipher<Decrypted> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        // Use a helper to avoid infinite recursion with flatten
        // This serializes the struct field by field without calling to_value
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct("Cipher", 26)?;

        // Serialize each field manually
        s.serialize_field("id", &self.id)?;
        s.serialize_field("organizationId", &self.organization_id)?;
        s.serialize_field("folderId", &self.folder_id)?;
        s.serialize_field("type", &self.r#type)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("notes", &self.notes)?;
        s.serialize_field("key", &self.key)?;
        s.serialize_field("favorite", &self.favorite)?;
        s.serialize_field("edit", &self.edit)?;
        s.serialize_field("viewPassword", &self.view_password)?;
        s.serialize_field("organizationUseTotp", &self.organization_use_totp)?;
        s.serialize_field("creationDate", &self.creation_date)?;
        s.serialize_field("revisionDate", &self.revision_date)?;
        s.serialize_field("deletedDate", &self.deleted_date)?;
        s.serialize_field("archivedDate", &self.archived_date)?;
        s.serialize_field("reprompt", &self.reprompt)?;
        s.serialize_field("permissions", &self.permissions)?;
        s.serialize_field("object", &self.object)?;
        s.serialize_field("fields", &self.fields)?;
        s.serialize_field("passwordHistory", &self.password_history)?;
        s.serialize_field("collectionIds", &self.collection_ids)?;
        s.serialize_field("data", &self.data)?;
        s.serialize_field("login", &self.login)?;
        s.serialize_field("secureNote", &self.secure_note)?;
        s.serialize_field("card", &self.card)?;
        s.serialize_field("identity", &self.identity)?;
        s.serialize_field("sshKey", &self.ssh_key)?;
        s.serialize_field("attachments", &self.attachments)?;

        s.end()
    }
}

/// Standard deserialization for Cipher<Decrypted> - uses camelCase
impl<'de> Deserialize<'de> for Cipher<Decrypted> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize with camelCase keys
        serde_json::Value::deserialize(deserializer)
            .and_then(|json| serde_json::from_value(json).map_err(::serde::de::Error::custom))
    }
}

/// Implement Decryptable for Cipher<Encrypted>
impl Decryptable for Cipher<Encrypted> {
    type Output = Cipher<Decrypted>;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        // Decrypt all encrypted fields
        let decrypted = Cipher {
            id: self.id,
            organization_id: self.organization_id,
            folder_id: self.folder_id,
            r#type: self.r#type,
            name: self.name.decrypt(key, &format!("{path}.name"))?,
            notes: self.notes.decrypt(key, &format!("{path}.notes"))?,
            key: self.key, // cipher.key is not decrypted like regular fields
            favorite: self.favorite,
            edit: self.edit,
            view_password: self.view_password,
            organization_use_totp: self.organization_use_totp,
            creation_date: self.creation_date,
            revision_date: self.revision_date,
            deleted_date: self.deleted_date,
            archived_date: self.archived_date,
            reprompt: self.reprompt,
            permissions: self.permissions,
            object: self.object,
            fields: self
                .fields
                .into_iter()
                .enumerate()
                .map(|(i, f)| {
                    let field_path = format!("{path}.fields[{}]", i);
                    decrypt_field(f, key, &field_path)
                })
                .collect::<Result<Vec<_>, AppError>>()?,
            password_history: self
                .password_history
                .into_iter()
                .enumerate()
                .map(|(i, h)| {
                    let history_path = format!("{path}.password_history[{}]", i);
                    decrypt_password_history(h, key, &history_path)
                })
                .collect::<Result<Vec<_>, AppError>>()?,
            collection_ids: self.collection_ids,
            data: self
                .data
                .map(|d| decrypt_data(d, key, &format!("{path}.data")))
                .transpose()?,
            login: self
                .login
                .map(|l| decrypt_login(l, key, &format!("{path}.login")))
                .transpose()?,
            secure_note: self.secure_note,
            card: self
                .card
                .map(|c| decrypt_card(c, key, &format!("{path}.card")))
                .transpose()?,
            identity: self
                .identity
                .map(|i| decrypt_identity(i, key, &format!("{path}.identity")))
                .transpose()?,
            ssh_key: self
                .ssh_key
                .map(|k| decrypt_ssh_key(k, key, &format!("{path}.ssh_key")))
                .transpose()?,
            attachments: self
                .attachments
                .into_iter()
                .enumerate()
                .map(|(i, a)| {
                    let attachment_path = format!("{path}.attachments[{}]", i);
                    decrypt_attachment(a, key, &attachment_path)
                })
                .collect::<Result<Vec<_>, AppError>>()?,
            _state: PhantomData,
        };

        Ok(decrypted)
    }
}

// Helper functions for decrypting nested structures

fn decrypt_field(
    field: CipherField<Encrypted>,
    key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<CipherField<Decrypted>, AppError> {
    Ok(CipherField {
        name: field.name.decrypt(key, &format!("{path}.name"))?,
        value: field.value.decrypt(key, &format!("{path}.value"))?,
        r#type: field.r#type,
        linked_id: field.linked_id,
    })
}

fn decrypt_password_history(
    history: CipherPasswordHistory<Encrypted>,
    key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<CipherPasswordHistory<Decrypted>, AppError> {
    Ok(CipherPasswordHistory {
        password: history.password.decrypt(key, &format!("{path}.password"))?,
        last_used_date: history.last_used_date,
    })
}

fn decrypt_data(
    data: CipherData<Encrypted>,
    key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<CipherData<Decrypted>, AppError> {
    // For now, we'll create a simplified implementation
    // A full implementation would decrypt all fields
    Ok(CipherData {
        name: data.name.decrypt(key, &format!("{path}.name"))?,
        notes: data.notes.decrypt(key, &format!("{path}.notes"))?,
        fields: data
            .fields
            .into_iter()
            .enumerate()
            .map(|(i, f)| {
                let field_path = format!("{path}.fields[{}]", i);
                decrypt_field(f, key, &field_path)
            })
            .collect::<Result<Vec<_>, AppError>>()?,
        password_history: data
            .password_history
            .into_iter()
            .enumerate()
            .map(|(i, h)| {
                let history_path = format!("{path}.password_history[{}]", i);
                decrypt_password_history(h, key, &history_path)
            })
            .collect::<Result<Vec<_>, AppError>>()?,
        uri: data.uri.decrypt(key, &format!("{path}.uri"))?,
        uris: data
            .uris
            .into_iter()
            .enumerate()
            .map(|(i, u)| {
                let uri_path = format!("{path}.uris[{}]", i);
                decrypt_uri(u, key, &uri_path)
            })
            .collect::<Result<Vec<_>, AppError>>()?,
        username: data.username.decrypt(key, &format!("{path}.username"))?,
        password: data.password.decrypt(key, &format!("{path}.password"))?,
        password_revision_date: data.password_revision_date,
        totp: data.totp.decrypt(key, &format!("{path}.totp"))?,
        autofill_on_page_load: data.autofill_on_page_load,
        fido2_credentials: data
            .fido2_credentials
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                let cred_path = format!("{path}.fido2_credentials[{}]", i);
                decrypt_fido2_credential(c, key, &cred_path)
            })
            .collect::<Result<Vec<_>, AppError>>()?,
        r#type: data.r#type,
        cardholder_name: data
            .cardholder_name
            .decrypt(key, &format!("{path}.cardholder_name"))?,
        brand: data.brand.decrypt(key, &format!("{path}.brand"))?,
        number: data.number.decrypt(key, &format!("{path}.number"))?,
        exp_month: data.exp_month.decrypt(key, &format!("{path}.exp_month"))?,
        exp_year: data.exp_year.decrypt(key, &format!("{path}.exp_year"))?,
        code: data.code.decrypt(key, &format!("{path}.code"))?,
        title: data.title.decrypt(key, &format!("{path}.title"))?,
        first_name: data
            .first_name
            .decrypt(key, &format!("{path}.first_name"))?,
        middle_name: data
            .middle_name
            .decrypt(key, &format!("{path}.middle_name"))?,
        last_name: data.last_name.decrypt(key, &format!("{path}.last_name"))?,
        address1: data.address1.decrypt(key, &format!("{path}.address1"))?,
        address2: data.address2.decrypt(key, &format!("{path}.address2"))?,
        address3: data.address3.decrypt(key, &format!("{path}.address3"))?,
        city: data.city.decrypt(key, &format!("{path}.city"))?,
        state: data.state.decrypt(key, &format!("{path}.state"))?,
        postal_code: data
            .postal_code
            .decrypt(key, &format!("{path}.postal_code"))?,
        country: data.country.decrypt(key, &format!("{path}.country"))?,
        company: data.company.decrypt(key, &format!("{path}.company"))?,
        email: data.email.decrypt(key, &format!("{path}.email"))?,
        phone: data.phone.decrypt(key, &format!("{path}.phone"))?,
        ssn: data.ssn.decrypt(key, &format!("{path}.ssn"))?,
        passport_number: data
            .passport_number
            .decrypt(key, &format!("{path}.passport_number"))?,
        license_number: data
            .license_number
            .decrypt(key, &format!("{path}.license_number"))?,
        private_key: data
            .private_key
            .decrypt(key, &format!("{path}.private_key"))?,
        public_key: data
            .public_key
            .decrypt(key, &format!("{path}.public_key"))?,
        key_fingerprint: data
            .key_fingerprint
            .decrypt(key, &format!("{path}.key_fingerprint"))?,
    })
}

fn decrypt_login(
    login: CipherLogin<Encrypted>,
    key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<CipherLogin<Decrypted>, AppError> {
    Ok(CipherLogin {
        uri: login.uri.decrypt(key, &format!("{path}.uri"))?,
        uris: login
            .uris
            .into_iter()
            .enumerate()
            .map(|(i, u)| {
                let uri_path = format!("{path}.uris[{}]", i);
                decrypt_uri(u, key, &uri_path)
            })
            .collect::<Result<Vec<_>, AppError>>()?,
        username: login.username.decrypt(key, &format!("{path}.username"))?,
        password: login.password.decrypt(key, &format!("{path}.password"))?,
        password_revision_date: login.password_revision_date,
        totp: login.totp.decrypt(key, &format!("{path}.totp"))?,
        autofill_on_page_load: login.autofill_on_page_load,
        fido2_credentials: login
            .fido2_credentials
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                let cred_path = format!("{path}.fido2_credentials[{}]", i);
                decrypt_fido2_credential(c, key, &cred_path)
            })
            .collect::<Result<Vec<_>, AppError>>()?,
    })
}

fn decrypt_uri(
    uri: CipherLoginUri<Encrypted>,
    key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<CipherLoginUri<Decrypted>, AppError> {
    Ok(CipherLoginUri {
        uri: uri.uri.decrypt(key, &format!("{path}.uri"))?,
        r#match: uri.r#match,
        uri_checksum: uri.uri_checksum,
    })
}

fn decrypt_fido2_credential(
    cred: CipherLoginFido2Credential<Encrypted>,
    key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<CipherLoginFido2Credential<Decrypted>, AppError> {
    Ok(CipherLoginFido2Credential {
        credential_id: cred
            .credential_id
            .decrypt(key, &format!("{path}.credential_id"))?,
        key_type: cred.key_type.decrypt(key, &format!("{path}.key_type"))?,
        key_algorithm: cred
            .key_algorithm
            .decrypt(key, &format!("{path}.key_algorithm"))?,
        key_curve: cred.key_curve.decrypt(key, &format!("{path}.key_curve"))?,
        key_value: cred.key_value.decrypt(key, &format!("{path}.key_value"))?,
        rp_id: cred.rp_id.decrypt(key, &format!("{path}.rp_id"))?,
        rp_name: cred.rp_name.decrypt(key, &format!("{path}.rp_name"))?,
        counter: cred.counter.decrypt(key, &format!("{path}.counter"))?,
        user_handle: cred
            .user_handle
            .decrypt(key, &format!("{path}.user_handle"))?,
        user_name: cred.user_name.decrypt(key, &format!("{path}.user_name"))?,
        user_display_name: cred
            .user_display_name
            .decrypt(key, &format!("{path}.user_display_name"))?,
        discoverable: cred
            .discoverable
            .decrypt(key, &format!("{path}.discoverable"))?,
        creation_date: cred
            .creation_date
            .decrypt(key, &format!("{path}.creation_date"))?,
    })
}

fn decrypt_card(
    card: CipherCard<Encrypted>,
    key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<CipherCard<Decrypted>, AppError> {
    Ok(CipherCard {
        cardholder_name: card
            .cardholder_name
            .decrypt(key, &format!("{path}.cardholder_name"))?,
        brand: card.brand.decrypt(key, &format!("{path}.brand"))?,
        number: card.number.decrypt(key, &format!("{path}.number"))?,
        exp_month: card.exp_month.decrypt(key, &format!("{path}.exp_month"))?,
        exp_year: card.exp_year.decrypt(key, &format!("{path}.exp_year"))?,
        code: card.code.decrypt(key, &format!("{path}.code"))?,
    })
}

fn decrypt_identity(
    identity: CipherIdentity<Encrypted>,
    key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<CipherIdentity<Decrypted>, AppError> {
    Ok(CipherIdentity {
        title: identity.title.decrypt(key, &format!("{path}.title"))?,
        first_name: identity
            .first_name
            .decrypt(key, &format!("{path}.first_name"))?,
        middle_name: identity
            .middle_name
            .decrypt(key, &format!("{path}.middle_name"))?,
        last_name: identity
            .last_name
            .decrypt(key, &format!("{path}.last_name"))?,
        address1: identity
            .address1
            .decrypt(key, &format!("{path}.address1"))?,
        address2: identity
            .address2
            .decrypt(key, &format!("{path}.address2"))?,
        address3: identity
            .address3
            .decrypt(key, &format!("{path}.address3"))?,
        city: identity.city.decrypt(key, &format!("{path}.city"))?,
        state: identity.state.decrypt(key, &format!("{path}.state"))?,
        postal_code: identity
            .postal_code
            .decrypt(key, &format!("{path}.postal_code"))?,
        country: identity.country.decrypt(key, &format!("{path}.country"))?,
        company: identity.company.decrypt(key, &format!("{path}.company"))?,
        email: identity.email.decrypt(key, &format!("{path}.email"))?,
        phone: identity.phone.decrypt(key, &format!("{path}.phone"))?,
        ssn: identity.ssn.decrypt(key, &format!("{path}.ssn"))?,
        username: identity
            .username
            .decrypt(key, &format!("{path}.username"))?,
        passport_number: identity
            .passport_number
            .decrypt(key, &format!("{path}.passport_number"))?,
        license_number: identity
            .license_number
            .decrypt(key, &format!("{path}.license_number"))?,
    })
}

fn decrypt_ssh_key(
    ssh_key: CipherSshKey<Encrypted>,
    key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<CipherSshKey<Decrypted>, AppError> {
    Ok(CipherSshKey {
        private_key: ssh_key
            .private_key
            .decrypt(key, &format!("{path}.private_key"))?,
        public_key: ssh_key
            .public_key
            .decrypt(key, &format!("{path}.public_key"))?,
        key_fingerprint: ssh_key
            .key_fingerprint
            .decrypt(key, &format!("{path}.key_fingerprint"))?,
    })
}

fn decrypt_attachment(
    attachment: CipherAttachment<Encrypted>,
    key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<CipherAttachment<Decrypted>, AppError> {
    Ok(CipherAttachment {
        id: attachment.id,
        key: attachment.key,
        file_name: attachment
            .file_name
            .decrypt(key, &format!("{path}.file_name"))?,
        size: attachment.size,
        size_name: attachment.size_name,
        url: attachment.url,
        object: attachment.object,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cipher_has_totp_with_login() {
        let cipher: Cipher<Encrypted> = Cipher {
            id: "test".to_string(),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: EncryptedField::new(Some("encrypted_name".to_string())),
            notes: EncryptedField::none(),
            key: None,
            favorite: None,
            edit: None,
            view_password: None,
            organization_use_totp: None,
            creation_date: None,
            revision_date: None,
            deleted_date: None,
            archived_date: None,
            reprompt: None,
            permissions: None,
            object: None,
            fields: vec![],
            password_history: vec![],
            collection_ids: vec![],
            data: None,
            login: Some(CipherLogin {
                uri: EncryptedField::none(),
                uris: vec![],
                username: EncryptedField::none(),
                password: EncryptedField::none(),
                password_revision_date: None,
                totp: EncryptedField::new(Some("encrypted_totp".to_string())),
                autofill_on_page_load: None,
                fido2_credentials: vec![],
            }),
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: vec![],
            _state: PhantomData,
        };

        assert!(cipher.has_totp());
    }

    #[test]
    fn test_cipher_has_totp_with_data() {
        let cipher: Cipher<Encrypted> = Cipher {
            id: "test".to_string(),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: EncryptedField::new(Some("encrypted_name".to_string())),
            notes: EncryptedField::none(),
            key: None,
            favorite: None,
            edit: None,
            view_password: None,
            organization_use_totp: None,
            creation_date: None,
            revision_date: None,
            deleted_date: None,
            archived_date: None,
            reprompt: None,
            permissions: None,
            object: None,
            fields: vec![],
            password_history: vec![],
            collection_ids: vec![],
            data: Some(CipherData {
                name: EncryptedField::new(Some("encrypted_name".to_string())),
                notes: EncryptedField::none(),
                fields: vec![],
                password_history: vec![],
                uri: EncryptedField::none(),
                uris: vec![],
                username: EncryptedField::none(),
                password: EncryptedField::none(),
                password_revision_date: None,
                totp: EncryptedField::new(Some("encrypted_totp".to_string())),
                autofill_on_page_load: None,
                fido2_credentials: vec![],
                r#type: None,
                cardholder_name: EncryptedField::none(),
                brand: EncryptedField::none(),
                number: EncryptedField::none(),
                exp_month: EncryptedField::none(),
                exp_year: EncryptedField::none(),
                code: EncryptedField::none(),
                title: EncryptedField::none(),
                first_name: EncryptedField::none(),
                middle_name: EncryptedField::none(),
                last_name: EncryptedField::none(),
                address1: EncryptedField::none(),
                address2: EncryptedField::none(),
                address3: EncryptedField::none(),
                city: EncryptedField::none(),
                state: EncryptedField::none(),
                postal_code: EncryptedField::none(),
                country: EncryptedField::none(),
                company: EncryptedField::none(),
                email: EncryptedField::none(),
                phone: EncryptedField::none(),
                ssn: EncryptedField::none(),
                passport_number: EncryptedField::none(),
                license_number: EncryptedField::none(),
                private_key: EncryptedField::none(),
                public_key: EncryptedField::none(),
                key_fingerprint: EncryptedField::none(),
            }),
            login: None,
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: vec![],
            _state: PhantomData,
        };

        assert!(cipher.has_totp());
    }

    #[test]
    fn test_cipher_has_totp_false() {
        let cipher: Cipher<Encrypted> = Cipher {
            id: "test".to_string(),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: EncryptedField::new(Some("encrypted_name".to_string())),
            notes: EncryptedField::none(),
            key: None,
            favorite: None,
            edit: None,
            view_password: None,
            organization_use_totp: None,
            creation_date: None,
            revision_date: None,
            deleted_date: None,
            archived_date: None,
            reprompt: None,
            permissions: None,
            object: None,
            fields: vec![],
            password_history: vec![],
            collection_ids: vec![],
            data: None,
            login: None,
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: vec![],
            _state: PhantomData,
        };

        assert!(!cipher.has_totp());
    }
}
