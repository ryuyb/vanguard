use crate::application::dto::sync::{
    SyncCipher, SyncCipherCard, SyncCipherField, SyncCipherIdentity, SyncCipherLogin,
    SyncCipherLoginUri, SyncCipherPasswordHistory, SyncCipherSshKey,
};
use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::application::vault_crypto;
use crate::support::error::AppError;
use crate::support::result::AppResult;

/// Encrypts all sensitive fields in a cipher before sending to the server
pub fn encrypt_cipher(
    cipher: &SyncCipher,
    user_key: &VaultUserKeyMaterial,
) -> AppResult<SyncCipher> {
    Ok(SyncCipher {
        id: cipher.id.clone(),
        organization_id: cipher.organization_id.clone(),
        folder_id: cipher.folder_id.clone(),
        r#type: cipher.r#type,
        name: encrypt_optional_field(&cipher.name, user_key, "cipher.name")?,
        notes: encrypt_optional_field(&cipher.notes, user_key, "cipher.notes")?,
        key: cipher.key.clone(),
        favorite: cipher.favorite,
        edit: cipher.edit,
        view_password: cipher.view_password,
        organization_use_totp: cipher.organization_use_totp,
        creation_date: cipher.creation_date.clone(),
        revision_date: cipher.revision_date.clone(),
        deleted_date: cipher.deleted_date.clone(),
        archived_date: cipher.archived_date.clone(),
        reprompt: cipher.reprompt,
        permissions: cipher.permissions.clone(),
        object: cipher.object.clone(),
        fields: encrypt_cipher_fields(&cipher.fields, user_key)?,
        password_history: encrypt_password_history(&cipher.password_history, user_key)?,
        collection_ids: cipher.collection_ids.clone(),
        data: cipher.data.clone(), // data is deprecated, not used
        login: cipher
            .login
            .as_ref()
            .map(|login| encrypt_cipher_login(login, user_key))
            .transpose()?,
        secure_note: cipher.secure_note.clone(),
        card: cipher
            .card
            .as_ref()
            .map(|card| encrypt_cipher_card(card, user_key))
            .transpose()?,
        identity: cipher
            .identity
            .as_ref()
            .map(|identity| encrypt_cipher_identity(identity, user_key))
            .transpose()?,
        ssh_key: cipher
            .ssh_key
            .as_ref()
            .map(|ssh_key| encrypt_cipher_ssh_key(ssh_key, user_key))
            .transpose()?,
        attachments: cipher.attachments.clone(), // attachments are handled separately
    })
}

fn encrypt_optional_field(
    value: &Option<String>,
    user_key: &VaultUserKeyMaterial,
    field_name: &str,
) -> AppResult<Option<String>> {
    match value {
        None => Ok(None),
        Some(plaintext) => {
            if plaintext.trim().is_empty() {
                return Ok(Some(plaintext.clone()));
            }

            // If already encrypted (starts with "0." or "2."), don't re-encrypt
            if vault_crypto::looks_like_cipher_string(plaintext) {
                return Ok(Some(plaintext.clone()));
            }

            vault_crypto::encrypt_cipher_string(plaintext, user_key)
                .map(Some)
                .map_err(|error| AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: format!(
                        "failed to encrypt field `{field_name}`: {}",
                        error.message()
                    ),
                })
        }
    }
}

fn encrypt_cipher_fields(
    fields: &[SyncCipherField],
    user_key: &VaultUserKeyMaterial,
) -> AppResult<Vec<SyncCipherField>> {
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let path = format!("cipher.fields[{index}]");
            Ok(SyncCipherField {
                name: encrypt_optional_field(&field.name, user_key, &format!("{path}.name"))?,
                value: encrypt_optional_field(&field.value, user_key, &format!("{path}.value"))?,
                r#type: field.r#type,
                linked_id: field.linked_id,
            })
        })
        .collect()
}

fn encrypt_password_history(
    history: &[SyncCipherPasswordHistory],
    user_key: &VaultUserKeyMaterial,
) -> AppResult<Vec<SyncCipherPasswordHistory>> {
    history
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let path = format!("cipher.password_history[{index}]");
            Ok(SyncCipherPasswordHistory {
                password: encrypt_optional_field(
                    &entry.password,
                    user_key,
                    &format!("{path}.password"),
                )?,
                last_used_date: entry.last_used_date.clone(),
            })
        })
        .collect()
}

fn encrypt_cipher_login(
    login: &SyncCipherLogin,
    user_key: &VaultUserKeyMaterial,
) -> AppResult<SyncCipherLogin> {
    Ok(SyncCipherLogin {
        uri: encrypt_optional_field(&login.uri, user_key, "cipher.login.uri")?,
        uris: login
            .uris
            .iter()
            .enumerate()
            .map(|(index, uri)| {
                Ok(SyncCipherLoginUri {
                    uri: encrypt_optional_field(
                        &uri.uri,
                        user_key,
                        &format!("cipher.login.uris[{index}].uri"),
                    )?,
                    r#match: uri.r#match,
                    uri_checksum: uri.uri_checksum.clone(),
                })
            })
            .collect::<AppResult<Vec<_>>>()?,
        username: encrypt_optional_field(&login.username, user_key, "cipher.login.username")?,
        password: encrypt_optional_field(&login.password, user_key, "cipher.login.password")?,
        password_revision_date: login.password_revision_date.clone(),
        totp: encrypt_optional_field(&login.totp, user_key, "cipher.login.totp")?,
        autofill_on_page_load: login.autofill_on_page_load,
        fido2_credentials: login.fido2_credentials.clone(), // FIDO2 credentials are already encrypted
    })
}

fn encrypt_cipher_card(
    card: &SyncCipherCard,
    user_key: &VaultUserKeyMaterial,
) -> AppResult<SyncCipherCard> {
    Ok(SyncCipherCard {
        cardholder_name: encrypt_optional_field(
            &card.cardholder_name,
            user_key,
            "cipher.card.cardholder_name",
        )?,
        brand: encrypt_optional_field(&card.brand, user_key, "cipher.card.brand")?,
        number: encrypt_optional_field(&card.number, user_key, "cipher.card.number")?,
        exp_month: encrypt_optional_field(&card.exp_month, user_key, "cipher.card.exp_month")?,
        exp_year: encrypt_optional_field(&card.exp_year, user_key, "cipher.card.exp_year")?,
        code: encrypt_optional_field(&card.code, user_key, "cipher.card.code")?,
    })
}

fn encrypt_cipher_identity(
    identity: &SyncCipherIdentity,
    user_key: &VaultUserKeyMaterial,
) -> AppResult<SyncCipherIdentity> {
    Ok(SyncCipherIdentity {
        title: encrypt_optional_field(&identity.title, user_key, "cipher.identity.title")?,
        first_name: encrypt_optional_field(
            &identity.first_name,
            user_key,
            "cipher.identity.first_name",
        )?,
        middle_name: encrypt_optional_field(
            &identity.middle_name,
            user_key,
            "cipher.identity.middle_name",
        )?,
        last_name: encrypt_optional_field(
            &identity.last_name,
            user_key,
            "cipher.identity.last_name",
        )?,
        address1: encrypt_optional_field(&identity.address1, user_key, "cipher.identity.address1")?,
        address2: encrypt_optional_field(&identity.address2, user_key, "cipher.identity.address2")?,
        address3: encrypt_optional_field(&identity.address3, user_key, "cipher.identity.address3")?,
        city: encrypt_optional_field(&identity.city, user_key, "cipher.identity.city")?,
        state: encrypt_optional_field(&identity.state, user_key, "cipher.identity.state")?,
        postal_code: encrypt_optional_field(
            &identity.postal_code,
            user_key,
            "cipher.identity.postal_code",
        )?,
        country: encrypt_optional_field(&identity.country, user_key, "cipher.identity.country")?,
        company: encrypt_optional_field(&identity.company, user_key, "cipher.identity.company")?,
        email: encrypt_optional_field(&identity.email, user_key, "cipher.identity.email")?,
        phone: encrypt_optional_field(&identity.phone, user_key, "cipher.identity.phone")?,
        ssn: encrypt_optional_field(&identity.ssn, user_key, "cipher.identity.ssn")?,
        username: encrypt_optional_field(&identity.username, user_key, "cipher.identity.username")?,
        passport_number: encrypt_optional_field(
            &identity.passport_number,
            user_key,
            "cipher.identity.passport_number",
        )?,
        license_number: encrypt_optional_field(
            &identity.license_number,
            user_key,
            "cipher.identity.license_number",
        )?,
    })
}

fn encrypt_cipher_ssh_key(
    ssh_key: &SyncCipherSshKey,
    user_key: &VaultUserKeyMaterial,
) -> AppResult<SyncCipherSshKey> {
    Ok(SyncCipherSshKey {
        private_key: encrypt_optional_field(
            &ssh_key.private_key,
            user_key,
            "cipher.ssh_key.private_key",
        )?,
        public_key: encrypt_optional_field(
            &ssh_key.public_key,
            user_key,
            "cipher.ssh_key.public_key",
        )?,
        key_fingerprint: encrypt_optional_field(
            &ssh_key.key_fingerprint,
            user_key,
            "cipher.ssh_key.key_fingerprint",
        )?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::sync::SyncCipherSecureNote;

    #[test]
    fn encrypt_cipher_encrypts_name_and_notes() {
        let user_key = VaultUserKeyMaterial {
            enc_key: vec![1u8; 32],
            mac_key: Some(vec![2u8; 32]),
        };

        let cipher = SyncCipher {
            id: String::from("cipher-1"),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: Some(String::from("My Login")),
            notes: Some(String::from("Secret notes")),
            key: None,
            favorite: Some(false),
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
            fields: Vec::new(),
            password_history: Vec::new(),
            collection_ids: Vec::new(),
            data: None,
            login: None,
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: Vec::new(),
        };

        let encrypted = encrypt_cipher(&cipher, &user_key).expect("encryption should succeed");

        // Name and notes should be encrypted (start with "2.")
        assert!(encrypted.name.as_ref().unwrap().starts_with("2."));
        assert!(encrypted.notes.as_ref().unwrap().starts_with("2."));

        // Verify we can decrypt them back
        let decrypted_name =
            vault_crypto::decrypt_cipher_string(encrypted.name.as_ref().unwrap(), &user_key)
                .expect("decrypt name");
        let decrypted_notes =
            vault_crypto::decrypt_cipher_string(encrypted.notes.as_ref().unwrap(), &user_key)
                .expect("decrypt notes");

        assert_eq!(decrypted_name, "My Login");
        assert_eq!(decrypted_notes, "Secret notes");
    }

    #[test]
    fn encrypt_cipher_encrypts_login_fields() {
        let user_key = VaultUserKeyMaterial {
            enc_key: vec![1u8; 32],
            mac_key: Some(vec![2u8; 32]),
        };

        let cipher = SyncCipher {
            id: String::from("cipher-1"),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: Some(String::from("Test")),
            notes: None,
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
            fields: Vec::new(),
            password_history: Vec::new(),
            collection_ids: Vec::new(),
            data: None,
            login: Some(SyncCipherLogin {
                uri: None,
                uris: vec![SyncCipherLoginUri {
                    uri: Some(String::from("https://example.com")),
                    r#match: Some(0),
                    uri_checksum: None,
                }],
                username: Some(String::from("user@example.com")),
                password: Some(String::from("secret123")),
                password_revision_date: None,
                totp: Some(String::from("JBSWY3DPEHPK3PXP")),
                autofill_on_page_load: None,
                fido2_credentials: Vec::new(),
            }),
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: Vec::new(),
        };

        let encrypted = encrypt_cipher(&cipher, &user_key).expect("encryption should succeed");

        let login = encrypted.login.as_ref().unwrap();
        assert!(login.username.as_ref().unwrap().starts_with("2."));
        assert!(login.password.as_ref().unwrap().starts_with("2."));
        assert!(login.totp.as_ref().unwrap().starts_with("2."));
        assert!(login.uris[0].uri.as_ref().unwrap().starts_with("2."));

        // Verify decryption
        let decrypted_username =
            vault_crypto::decrypt_cipher_string(login.username.as_ref().unwrap(), &user_key)
                .expect("decrypt username");
        let decrypted_password =
            vault_crypto::decrypt_cipher_string(login.password.as_ref().unwrap(), &user_key)
                .expect("decrypt password");

        assert_eq!(decrypted_username, "user@example.com");
        assert_eq!(decrypted_password, "secret123");
    }

    #[test]
    fn encrypt_cipher_skips_already_encrypted_fields() {
        let user_key = VaultUserKeyMaterial {
            enc_key: vec![1u8; 32],
            mac_key: Some(vec![2u8; 32]),
        };

        // Pre-encrypt a value
        let encrypted_name =
            vault_crypto::encrypt_cipher_string("Already Encrypted", &user_key).unwrap();

        let cipher = SyncCipher {
            id: String::from("cipher-1"),
            organization_id: None,
            folder_id: None,
            r#type: Some(2),
            name: Some(encrypted_name.clone()),
            notes: None,
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
            fields: Vec::new(),
            password_history: Vec::new(),
            collection_ids: Vec::new(),
            data: None,
            login: None,
            secure_note: Some(SyncCipherSecureNote { r#type: Some(0) }),
            card: None,
            identity: None,
            ssh_key: None,
            attachments: Vec::new(),
        };

        let encrypted = encrypt_cipher(&cipher, &user_key).expect("encryption should succeed");

        // Should not re-encrypt
        assert_eq!(encrypted.name.as_ref().unwrap(), &encrypted_name);
    }

    #[test]
    fn encrypt_cipher_handles_empty_strings() {
        let user_key = VaultUserKeyMaterial {
            enc_key: vec![1u8; 32],
            mac_key: Some(vec![2u8; 32]),
        };

        let cipher = SyncCipher {
            id: String::from("cipher-1"),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: Some(String::from("")),
            notes: Some(String::from("   ")),
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
            fields: Vec::new(),
            password_history: Vec::new(),
            collection_ids: Vec::new(),
            data: None,
            login: None,
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: Vec::new(),
        };

        let encrypted = encrypt_cipher(&cipher, &user_key).expect("encryption should succeed");

        // Empty strings should not be encrypted
        assert_eq!(encrypted.name.as_ref().unwrap(), "");
        assert_eq!(encrypted.notes.as_ref().unwrap(), "   ");
    }
}
