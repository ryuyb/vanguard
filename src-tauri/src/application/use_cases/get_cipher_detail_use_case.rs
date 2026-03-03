use std::sync::Arc;

use crate::application::dto::sync::{
    SyncAttachment, SyncCipher, SyncCipherCard, SyncCipherData, SyncCipherField,
    SyncCipherIdentity, SyncCipherLogin, SyncCipherLoginFido2Credential, SyncCipherLoginUri,
    SyncCipherPasswordHistory, SyncCipherSshKey,
};
use crate::application::dto::vault::{
    GetCipherDetailQuery, VaultAttachmentDetail, VaultCipherCardDetail, VaultCipherDataDetail,
    VaultCipherDetail, VaultCipherFieldDetail, VaultCipherIdentityDetail, VaultCipherLoginDetail,
    VaultCipherLoginFido2CredentialDetail, VaultCipherLoginUriDetail,
    VaultCipherPasswordHistoryDetail, VaultCipherPermissionsDetail, VaultCipherSecureNoteDetail,
    VaultCipherSshKeyDetail, VaultUserKeyMaterial,
};
use crate::application::services::sync_service::SyncService;
use crate::application::vault_crypto;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct GetCipherDetailUseCase {
    sync_service: Arc<SyncService>,
}

impl GetCipherDetailUseCase {
    pub fn new(sync_service: Arc<SyncService>) -> Self {
        Self { sync_service }
    }

    pub async fn execute(&self, query: GetCipherDetailQuery) -> AppResult<VaultCipherDetail> {
        require_non_empty(&query.account_id, "account_id")?;
        require_non_empty(&query.cipher_id, "cipher_id")?;
        vault_crypto::validate_key_lengths(
            &query.user_key.enc_key,
            query.user_key.mac_key.as_deref(),
        )?;

        let cipher = self
            .sync_service
            .get_live_cipher(query.account_id.clone(), query.cipher_id.clone())
            .await?
            .ok_or_else(|| {
                AppError::validation(format!("cipher not found: {}", query.cipher_id))
            })?;

        decrypt_cipher_detail(cipher, &query.user_key)
    }
}

fn decrypt_cipher_detail(
    cipher: SyncCipher,
    user_key: &VaultUserKeyMaterial,
) -> Result<VaultCipherDetail, AppError> {
    let detail_key = resolve_cipher_decryption_key(cipher.key.as_deref(), user_key)?;

    Ok(VaultCipherDetail {
        id: cipher.id,
        organization_id: cipher.organization_id,
        folder_id: cipher.folder_id,
        r#type: cipher.r#type,
        name: vault_crypto::decrypt_optional_field(cipher.name, &detail_key, "cipher.name")?,
        notes: vault_crypto::decrypt_optional_field(cipher.notes, &detail_key, "cipher.notes")?,
        key: cipher.key,
        favorite: cipher.favorite,
        edit: cipher.edit,
        view_password: cipher.view_password,
        organization_use_totp: cipher.organization_use_totp,
        creation_date: cipher.creation_date,
        revision_date: cipher.revision_date,
        deleted_date: cipher.deleted_date,
        archived_date: cipher.archived_date,
        reprompt: cipher.reprompt,
        permissions: cipher
            .permissions
            .map(|permissions| VaultCipherPermissionsDetail {
                delete: permissions.delete,
                restore: permissions.restore,
            }),
        object: cipher.object,
        fields: decrypt_cipher_fields(cipher.fields, &detail_key, "cipher.fields")?,
        password_history: decrypt_password_history(
            cipher.password_history,
            &detail_key,
            "cipher.password_history",
        )?,
        collection_ids: cipher.collection_ids,
        data: decrypt_cipher_data_detail(cipher.data, &detail_key)?,
        login: decrypt_cipher_login_detail(cipher.login, &detail_key)?,
        secure_note: cipher.secure_note.map(|note| VaultCipherSecureNoteDetail {
            r#type: note.r#type,
        }),
        card: decrypt_cipher_card_detail(cipher.card, &detail_key)?,
        identity: decrypt_cipher_identity_detail(cipher.identity, &detail_key)?,
        ssh_key: decrypt_cipher_ssh_key_detail(cipher.ssh_key, &detail_key)?,
        attachments: decrypt_attachments(cipher.attachments, &detail_key)?,
    })
}

fn resolve_cipher_decryption_key(
    cipher_key: Option<&str>,
    user_key: &VaultUserKeyMaterial,
) -> Result<VaultUserKeyMaterial, AppError> {
    let Some(raw) = cipher_key else {
        return Ok(user_key.clone());
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(user_key.clone());
    }

    if vault_crypto::looks_like_cipher_string(trimmed) {
        let decrypted = vault_crypto::decrypt_cipher_bytes(trimmed, user_key).map_err(|error| {
            AppError::validation(format!(
                "failed to decrypt field `cipher.key`: {}",
                error.message()
            ))
        })?;
        return vault_crypto::parse_user_key_material(&decrypted).map_err(|error| {
            AppError::validation(format!(
                "failed to parse decrypted field `cipher.key`: {}",
                error.message()
            ))
        });
    }

    vault_crypto::parse_user_key(trimmed).map_err(|error| {
        AppError::validation(format!(
            "failed to parse field `cipher.key`: {}",
            error.message()
        ))
    })
}

fn decrypt_cipher_fields(
    fields: Vec<SyncCipherField>,
    user_key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<Vec<VaultCipherFieldDetail>, AppError> {
    fields
        .into_iter()
        .enumerate()
        .map(|(index, field)| {
            let entry_path = format!("{path}[{index}]");
            Ok(VaultCipherFieldDetail {
                name: vault_crypto::decrypt_optional_field(
                    field.name,
                    user_key,
                    &format!("{entry_path}.name"),
                )?,
                value: vault_crypto::decrypt_optional_field(
                    field.value,
                    user_key,
                    &format!("{entry_path}.value"),
                )?,
                r#type: field.r#type,
                linked_id: field.linked_id,
            })
        })
        .collect()
}

fn decrypt_password_history(
    entries: Vec<SyncCipherPasswordHistory>,
    user_key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<Vec<VaultCipherPasswordHistoryDetail>, AppError> {
    entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let entry_path = format!("{path}[{index}]");
            Ok(VaultCipherPasswordHistoryDetail {
                password: vault_crypto::decrypt_optional_field(
                    entry.password,
                    user_key,
                    &format!("{entry_path}.password"),
                )?,
                last_used_date: entry.last_used_date,
            })
        })
        .collect()
}

fn decrypt_attachments(
    attachments: Vec<SyncAttachment>,
    user_key: &VaultUserKeyMaterial,
) -> Result<Vec<VaultAttachmentDetail>, AppError> {
    attachments
        .into_iter()
        .enumerate()
        .map(|(index, attachment)| {
            let entry_path = format!("cipher.attachments[{index}]");
            Ok(VaultAttachmentDetail {
                id: attachment.id,
                key: attachment.key,
                file_name: vault_crypto::decrypt_optional_field(
                    attachment.file_name,
                    user_key,
                    &format!("{entry_path}.file_name"),
                )?,
                size: attachment.size,
                size_name: attachment.size_name,
                url: attachment.url,
                object: attachment.object,
            })
        })
        .collect()
}

fn decrypt_cipher_data_detail(
    data: Option<SyncCipherData>,
    user_key: &VaultUserKeyMaterial,
) -> Result<Option<VaultCipherDataDetail>, AppError> {
    data.map(|entry| {
        Ok(VaultCipherDataDetail {
            name: vault_crypto::decrypt_optional_field(entry.name, user_key, "cipher.data.name")?,
            notes: vault_crypto::decrypt_optional_field(
                entry.notes,
                user_key,
                "cipher.data.notes",
            )?,
            fields: decrypt_cipher_fields(entry.fields, user_key, "cipher.data.fields")?,
            password_history: decrypt_password_history(
                entry.password_history,
                user_key,
                "cipher.data.password_history",
            )?,
            uri: vault_crypto::decrypt_optional_field(entry.uri, user_key, "cipher.data.uri")?,
            uris: decrypt_login_uris(entry.uris, user_key, "cipher.data.uris")?,
            username: vault_crypto::decrypt_optional_field(
                entry.username,
                user_key,
                "cipher.data.username",
            )?,
            password: vault_crypto::decrypt_optional_field(
                entry.password,
                user_key,
                "cipher.data.password",
            )?,
            password_revision_date: entry.password_revision_date,
            totp: vault_crypto::decrypt_optional_field(entry.totp, user_key, "cipher.data.totp")?,
            autofill_on_page_load: entry.autofill_on_page_load,
            fido2_credentials: decrypt_fido2_credentials(
                entry.fido2_credentials,
                user_key,
                "cipher.data.fido2_credentials",
            )?,
            r#type: entry.r#type,
            cardholder_name: vault_crypto::decrypt_optional_field(
                entry.cardholder_name,
                user_key,
                "cipher.data.cardholder_name",
            )?,
            brand: vault_crypto::decrypt_optional_field(
                entry.brand,
                user_key,
                "cipher.data.brand",
            )?,
            number: vault_crypto::decrypt_optional_field(
                entry.number,
                user_key,
                "cipher.data.number",
            )?,
            exp_month: vault_crypto::decrypt_optional_field(
                entry.exp_month,
                user_key,
                "cipher.data.exp_month",
            )?,
            exp_year: vault_crypto::decrypt_optional_field(
                entry.exp_year,
                user_key,
                "cipher.data.exp_year",
            )?,
            code: vault_crypto::decrypt_optional_field(entry.code, user_key, "cipher.data.code")?,
            title: vault_crypto::decrypt_optional_field(
                entry.title,
                user_key,
                "cipher.data.title",
            )?,
            first_name: vault_crypto::decrypt_optional_field(
                entry.first_name,
                user_key,
                "cipher.data.first_name",
            )?,
            middle_name: vault_crypto::decrypt_optional_field(
                entry.middle_name,
                user_key,
                "cipher.data.middle_name",
            )?,
            last_name: vault_crypto::decrypt_optional_field(
                entry.last_name,
                user_key,
                "cipher.data.last_name",
            )?,
            address1: vault_crypto::decrypt_optional_field(
                entry.address1,
                user_key,
                "cipher.data.address1",
            )?,
            address2: vault_crypto::decrypt_optional_field(
                entry.address2,
                user_key,
                "cipher.data.address2",
            )?,
            address3: vault_crypto::decrypt_optional_field(
                entry.address3,
                user_key,
                "cipher.data.address3",
            )?,
            city: vault_crypto::decrypt_optional_field(entry.city, user_key, "cipher.data.city")?,
            state: vault_crypto::decrypt_optional_field(
                entry.state,
                user_key,
                "cipher.data.state",
            )?,
            postal_code: vault_crypto::decrypt_optional_field(
                entry.postal_code,
                user_key,
                "cipher.data.postal_code",
            )?,
            country: vault_crypto::decrypt_optional_field(
                entry.country,
                user_key,
                "cipher.data.country",
            )?,
            company: vault_crypto::decrypt_optional_field(
                entry.company,
                user_key,
                "cipher.data.company",
            )?,
            email: vault_crypto::decrypt_optional_field(
                entry.email,
                user_key,
                "cipher.data.email",
            )?,
            phone: vault_crypto::decrypt_optional_field(
                entry.phone,
                user_key,
                "cipher.data.phone",
            )?,
            ssn: vault_crypto::decrypt_optional_field(entry.ssn, user_key, "cipher.data.ssn")?,
            passport_number: vault_crypto::decrypt_optional_field(
                entry.passport_number,
                user_key,
                "cipher.data.passport_number",
            )?,
            license_number: vault_crypto::decrypt_optional_field(
                entry.license_number,
                user_key,
                "cipher.data.license_number",
            )?,
            private_key: vault_crypto::decrypt_optional_field(
                entry.private_key,
                user_key,
                "cipher.data.private_key",
            )?,
            public_key: vault_crypto::decrypt_optional_field(
                entry.public_key,
                user_key,
                "cipher.data.public_key",
            )?,
            key_fingerprint: vault_crypto::decrypt_optional_field(
                entry.key_fingerprint,
                user_key,
                "cipher.data.key_fingerprint",
            )?,
        })
    })
    .transpose()
}

fn decrypt_cipher_login_detail(
    login: Option<SyncCipherLogin>,
    user_key: &VaultUserKeyMaterial,
) -> Result<Option<VaultCipherLoginDetail>, AppError> {
    login
        .map(|entry| {
            Ok(VaultCipherLoginDetail {
                uri: vault_crypto::decrypt_optional_field(entry.uri, user_key, "cipher.login.uri")?,
                uris: decrypt_login_uris(entry.uris, user_key, "cipher.login.uris")?,
                username: vault_crypto::decrypt_optional_field(
                    entry.username,
                    user_key,
                    "cipher.login.username",
                )?,
                password: vault_crypto::decrypt_optional_field(
                    entry.password,
                    user_key,
                    "cipher.login.password",
                )?,
                password_revision_date: entry.password_revision_date,
                totp: vault_crypto::decrypt_optional_field(
                    entry.totp,
                    user_key,
                    "cipher.login.totp",
                )?,
                autofill_on_page_load: entry.autofill_on_page_load,
                fido2_credentials: decrypt_fido2_credentials(
                    entry.fido2_credentials,
                    user_key,
                    "cipher.login.fido2_credentials",
                )?,
            })
        })
        .transpose()
}

fn decrypt_login_uris(
    uris: Vec<SyncCipherLoginUri>,
    user_key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<Vec<VaultCipherLoginUriDetail>, AppError> {
    uris.into_iter()
        .enumerate()
        .map(|(index, uri)| {
            let entry_path = format!("{path}[{index}]");
            Ok(VaultCipherLoginUriDetail {
                uri: vault_crypto::decrypt_optional_field(
                    uri.uri,
                    user_key,
                    &format!("{entry_path}.uri"),
                )?,
                r#match: uri.r#match,
                uri_checksum: uri.uri_checksum,
            })
        })
        .collect()
}

fn decrypt_fido2_credentials(
    credentials: Vec<SyncCipherLoginFido2Credential>,
    user_key: &VaultUserKeyMaterial,
    path: &str,
) -> Result<Vec<VaultCipherLoginFido2CredentialDetail>, AppError> {
    credentials
        .into_iter()
        .enumerate()
        .map(|(index, credential)| {
            let entry_path = format!("{path}[{index}]");
            Ok(VaultCipherLoginFido2CredentialDetail {
                credential_id: vault_crypto::decrypt_optional_field(
                    credential.credential_id,
                    user_key,
                    &format!("{entry_path}.credential_id"),
                )?,
                key_type: vault_crypto::decrypt_optional_field(
                    credential.key_type,
                    user_key,
                    &format!("{entry_path}.key_type"),
                )?,
                key_algorithm: vault_crypto::decrypt_optional_field(
                    credential.key_algorithm,
                    user_key,
                    &format!("{entry_path}.key_algorithm"),
                )?,
                key_curve: vault_crypto::decrypt_optional_field(
                    credential.key_curve,
                    user_key,
                    &format!("{entry_path}.key_curve"),
                )?,
                key_value: vault_crypto::decrypt_optional_field(
                    credential.key_value,
                    user_key,
                    &format!("{entry_path}.key_value"),
                )?,
                rp_id: vault_crypto::decrypt_optional_field(
                    credential.rp_id,
                    user_key,
                    &format!("{entry_path}.rp_id"),
                )?,
                rp_name: vault_crypto::decrypt_optional_field(
                    credential.rp_name,
                    user_key,
                    &format!("{entry_path}.rp_name"),
                )?,
                counter: vault_crypto::decrypt_optional_field(
                    credential.counter,
                    user_key,
                    &format!("{entry_path}.counter"),
                )?,
                user_handle: vault_crypto::decrypt_optional_field(
                    credential.user_handle,
                    user_key,
                    &format!("{entry_path}.user_handle"),
                )?,
                user_name: vault_crypto::decrypt_optional_field(
                    credential.user_name,
                    user_key,
                    &format!("{entry_path}.user_name"),
                )?,
                user_display_name: vault_crypto::decrypt_optional_field(
                    credential.user_display_name,
                    user_key,
                    &format!("{entry_path}.user_display_name"),
                )?,
                discoverable: vault_crypto::decrypt_optional_field(
                    credential.discoverable,
                    user_key,
                    &format!("{entry_path}.discoverable"),
                )?,
                creation_date: vault_crypto::decrypt_optional_field(
                    credential.creation_date,
                    user_key,
                    &format!("{entry_path}.creation_date"),
                )?,
            })
        })
        .collect()
}

fn decrypt_cipher_card_detail(
    card: Option<SyncCipherCard>,
    user_key: &VaultUserKeyMaterial,
) -> Result<Option<VaultCipherCardDetail>, AppError> {
    card.map(|entry| {
        Ok(VaultCipherCardDetail {
            cardholder_name: vault_crypto::decrypt_optional_field(
                entry.cardholder_name,
                user_key,
                "cipher.card.cardholder_name",
            )?,
            brand: vault_crypto::decrypt_optional_field(
                entry.brand,
                user_key,
                "cipher.card.brand",
            )?,
            number: vault_crypto::decrypt_optional_field(
                entry.number,
                user_key,
                "cipher.card.number",
            )?,
            exp_month: vault_crypto::decrypt_optional_field(
                entry.exp_month,
                user_key,
                "cipher.card.exp_month",
            )?,
            exp_year: vault_crypto::decrypt_optional_field(
                entry.exp_year,
                user_key,
                "cipher.card.exp_year",
            )?,
            code: vault_crypto::decrypt_optional_field(entry.code, user_key, "cipher.card.code")?,
        })
    })
    .transpose()
}

fn decrypt_cipher_identity_detail(
    identity: Option<SyncCipherIdentity>,
    user_key: &VaultUserKeyMaterial,
) -> Result<Option<VaultCipherIdentityDetail>, AppError> {
    identity
        .map(|entry| {
            Ok(VaultCipherIdentityDetail {
                title: vault_crypto::decrypt_optional_field(
                    entry.title,
                    user_key,
                    "cipher.identity.title",
                )?,
                first_name: vault_crypto::decrypt_optional_field(
                    entry.first_name,
                    user_key,
                    "cipher.identity.first_name",
                )?,
                middle_name: vault_crypto::decrypt_optional_field(
                    entry.middle_name,
                    user_key,
                    "cipher.identity.middle_name",
                )?,
                last_name: vault_crypto::decrypt_optional_field(
                    entry.last_name,
                    user_key,
                    "cipher.identity.last_name",
                )?,
                address1: vault_crypto::decrypt_optional_field(
                    entry.address1,
                    user_key,
                    "cipher.identity.address1",
                )?,
                address2: vault_crypto::decrypt_optional_field(
                    entry.address2,
                    user_key,
                    "cipher.identity.address2",
                )?,
                address3: vault_crypto::decrypt_optional_field(
                    entry.address3,
                    user_key,
                    "cipher.identity.address3",
                )?,
                city: vault_crypto::decrypt_optional_field(
                    entry.city,
                    user_key,
                    "cipher.identity.city",
                )?,
                state: vault_crypto::decrypt_optional_field(
                    entry.state,
                    user_key,
                    "cipher.identity.state",
                )?,
                postal_code: vault_crypto::decrypt_optional_field(
                    entry.postal_code,
                    user_key,
                    "cipher.identity.postal_code",
                )?,
                country: vault_crypto::decrypt_optional_field(
                    entry.country,
                    user_key,
                    "cipher.identity.country",
                )?,
                company: vault_crypto::decrypt_optional_field(
                    entry.company,
                    user_key,
                    "cipher.identity.company",
                )?,
                email: vault_crypto::decrypt_optional_field(
                    entry.email,
                    user_key,
                    "cipher.identity.email",
                )?,
                phone: vault_crypto::decrypt_optional_field(
                    entry.phone,
                    user_key,
                    "cipher.identity.phone",
                )?,
                ssn: vault_crypto::decrypt_optional_field(
                    entry.ssn,
                    user_key,
                    "cipher.identity.ssn",
                )?,
                username: vault_crypto::decrypt_optional_field(
                    entry.username,
                    user_key,
                    "cipher.identity.username",
                )?,
                passport_number: vault_crypto::decrypt_optional_field(
                    entry.passport_number,
                    user_key,
                    "cipher.identity.passport_number",
                )?,
                license_number: vault_crypto::decrypt_optional_field(
                    entry.license_number,
                    user_key,
                    "cipher.identity.license_number",
                )?,
            })
        })
        .transpose()
}

fn decrypt_cipher_ssh_key_detail(
    ssh_key: Option<SyncCipherSshKey>,
    user_key: &VaultUserKeyMaterial,
) -> Result<Option<VaultCipherSshKeyDetail>, AppError> {
    ssh_key
        .map(|entry| {
            Ok(VaultCipherSshKeyDetail {
                private_key: vault_crypto::decrypt_optional_field(
                    entry.private_key,
                    user_key,
                    "cipher.ssh_key.private_key",
                )?,
                public_key: vault_crypto::decrypt_optional_field(
                    entry.public_key,
                    user_key,
                    "cipher.ssh_key.public_key",
                )?,
                key_fingerprint: vault_crypto::decrypt_optional_field(
                    entry.key_fingerprint,
                    user_key,
                    "cipher.ssh_key.key_fingerprint",
                )?,
            })
        })
        .transpose()
}

fn require_non_empty(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::validation(format!("{field} cannot be empty")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes::Aes256;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use cbc::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type Aes256CbcEncryptor = cbc::Encryptor<Aes256>;
    type HmacSha256 = Hmac<Sha256>;

    #[test]
    fn decrypt_type2_cipher_string_roundtrip() {
        let enc_key = [1u8; 32];
        let mac_key = [2u8; 32];
        let key = VaultUserKeyMaterial {
            enc_key: enc_key.to_vec(),
            mac_key: Some(mac_key.to_vec()),
        };

        let cipher = encrypt_type2("hello-vault", &enc_key, &mac_key);
        let plaintext = vault_crypto::decrypt_cipher_string(&cipher, &key).expect("decrypt type2");
        assert_eq!(plaintext, "hello-vault");
    }

    #[test]
    fn decrypt_type2_without_mac_key_is_rejected() {
        let enc_key = [1u8; 32];
        let mac_key = [2u8; 32];
        let key = VaultUserKeyMaterial {
            enc_key: enc_key.to_vec(),
            mac_key: None,
        };

        let cipher = encrypt_type2("hello-vault", &enc_key, &mac_key);
        let error = vault_crypto::decrypt_cipher_string(&cipher, &key).expect_err("must fail");
        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn decrypt_cipher_detail_decrypts_whitelisted_fields_only() {
        let enc_key = [1u8; 32];
        let mac_key = [2u8; 32];
        let user_key = VaultUserKeyMaterial {
            enc_key: enc_key.to_vec(),
            mac_key: Some(mac_key.to_vec()),
        };
        let encrypted_name = encrypt_type2("demo", &enc_key, &mac_key);
        let encrypted_password = encrypt_type2("history-pass", &enc_key, &mac_key);
        let encrypted_login_uri = encrypt_type2("https://example.com/login", &enc_key, &mac_key);
        let cipher = SyncCipher {
            id: String::from("cipher-1"),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: Some(encrypted_name),
            notes: Some(String::from("note")),
            key: None,
            favorite: Some(false),
            edit: Some(true),
            view_password: Some(true),
            organization_use_totp: Some(false),
            creation_date: Some(String::from("2026-03-01T00:00:00Z")),
            revision_date: Some(String::from("2026-03-01T00:00:00Z")),
            deleted_date: None,
            archived_date: None,
            reprompt: Some(0),
            permissions: None,
            object: Some(String::from("cipher")),
            fields: Vec::new(),
            password_history: vec![SyncCipherPasswordHistory {
                password: Some(encrypted_password),
                last_used_date: Some(String::from("2026-03-01T00:00:00Z")),
            }],
            collection_ids: Vec::new(),
            data: None,
            login: Some(SyncCipherLogin {
                uri: None,
                uris: vec![SyncCipherLoginUri {
                    uri: Some(encrypted_login_uri),
                    r#match: Some(0),
                    uri_checksum: Some(String::from("2.not-a-cipher|string|shape")),
                }],
                username: None,
                password: None,
                password_revision_date: None,
                totp: None,
                autofill_on_page_load: Some(false),
                fido2_credentials: Vec::new(),
            }),
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: Vec::new(),
        };

        let detail = decrypt_cipher_detail(cipher, &user_key).expect("detail deserialize");
        assert_eq!(detail.id, "cipher-1");
        assert_eq!(detail.name.as_deref(), Some("demo"));
        assert_eq!(detail.key, None);
        assert_eq!(detail.password_history.len(), 1);
        assert_eq!(
            detail.password_history[0].password.as_deref(),
            Some("history-pass")
        );
        assert_eq!(
            detail.password_history[0].last_used_date.as_deref(),
            Some("2026-03-01T00:00:00Z")
        );
        assert_eq!(
            detail.login.as_ref().expect("login").uris[0].uri.as_deref(),
            Some("https://example.com/login")
        );
        assert_eq!(
            detail.login.as_ref().expect("login").uris[0]
                .uri_checksum
                .as_deref(),
            Some("2.not-a-cipher|string|shape")
        );
    }

    #[test]
    fn decrypt_cipher_detail_uses_cipher_key_for_field_decryption() {
        let user_enc_key = [1u8; 32];
        let user_mac_key = [2u8; 32];
        let user_key = VaultUserKeyMaterial {
            enc_key: user_enc_key.to_vec(),
            mac_key: Some(user_mac_key.to_vec()),
        };

        let cipher_enc_key = [3u8; 32];
        let cipher_mac_key = [4u8; 32];
        let mut cipher_key_material = Vec::with_capacity(64);
        cipher_key_material.extend_from_slice(&cipher_enc_key);
        cipher_key_material.extend_from_slice(&cipher_mac_key);
        let plain_cipher_key = STANDARD.encode(&cipher_key_material);

        let encrypted_cipher_key = encrypt_type2(&plain_cipher_key, &user_enc_key, &user_mac_key);
        let encrypted_name = encrypt_type2("cipher-name", &cipher_enc_key, &cipher_mac_key);
        let encrypted_password = encrypt_type2("cipher-pass", &cipher_enc_key, &cipher_mac_key);

        let cipher = SyncCipher {
            id: String::from("cipher-key-1"),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: Some(encrypted_name),
            notes: None,
            key: Some(encrypted_cipher_key.clone()),
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
            password_history: vec![SyncCipherPasswordHistory {
                password: Some(encrypted_password),
                last_used_date: None,
            }],
            collection_ids: Vec::new(),
            data: None,
            login: None,
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: Vec::new(),
        };

        let detail = decrypt_cipher_detail(cipher, &user_key).expect("detail decrypt");
        assert_eq!(detail.name.as_deref(), Some("cipher-name"));
        assert_eq!(
            detail.password_history[0].password.as_deref(),
            Some("cipher-pass")
        );
        assert_eq!(detail.key.as_deref(), Some(encrypted_cipher_key.as_str()));
    }

    #[test]
    fn decrypt_cipher_detail_rejects_invalid_cipher_on_whitelisted_field() {
        let user_key = VaultUserKeyMaterial {
            enc_key: [1u8; 32].to_vec(),
            mac_key: Some([2u8; 32].to_vec()),
        };
        let cipher = SyncCipher {
            id: String::from("cipher-1"),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: Some(String::from("2.not-base64|still-not-base64|bad-mac")),
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
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: Vec::new(),
        };

        let error = decrypt_cipher_detail(cipher, &user_key)
            .expect_err("invalid encrypted field must fail");
        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn decrypt_cipher_detail_rejects_invalid_cipher_key() {
        let user_key = VaultUserKeyMaterial {
            enc_key: [1u8; 32].to_vec(),
            mac_key: Some([2u8; 32].to_vec()),
        };
        let cipher = SyncCipher {
            id: String::from("cipher-1"),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: Some(String::from("plain-name")),
            notes: None,
            key: Some(String::from("2.not-base64|still-not-base64|bad-mac")),
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

        let error =
            decrypt_cipher_detail(cipher, &user_key).expect_err("invalid cipher key must fail");
        assert_eq!(error.code(), "validation_error");
    }

    fn encrypt_type2(plaintext: &str, enc_key: &[u8; 32], mac_key: &[u8; 32]) -> String {
        encrypt_type2_bytes(plaintext.as_bytes(), enc_key, mac_key)
    }

    fn encrypt_type2_bytes(plaintext: &[u8], enc_key: &[u8], mac_key: &[u8]) -> String {
        let iv = [9u8; 16];
        let mut buffer = plaintext.to_vec();
        let message_len = buffer.len();
        buffer.resize(message_len + 16, 0);

        let ciphertext = Aes256CbcEncryptor::new_from_slices(enc_key, &iv)
            .expect("build encryptor")
            .encrypt_padded_mut::<Pkcs7>(&mut buffer, message_len)
            .expect("encrypt")
            .to_vec();

        let mut mac = HmacSha256::new_from_slice(mac_key).expect("build hmac");
        mac.update(&iv);
        mac.update(&ciphertext);
        let mac = mac.finalize().into_bytes();

        format!(
            "2.{}|{}|{}",
            STANDARD.encode(iv),
            STANDARD.encode(ciphertext),
            STANDARD.encode(mac)
        )
    }
}
