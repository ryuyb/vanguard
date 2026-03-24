use crate::application::dto::sync::SyncCipher;
use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::domain::crypto::Decryptable;
use crate::support::error::AppError;

use super::field::EncryptedField;
use super::state::{Decrypted, Encrypted};
use super::Cipher;

/// Implement Decryptable for EncryptedField<Encrypted, String>
impl Decryptable for EncryptedField<Encrypted, String> {
    type Output = EncryptedField<Decrypted, String>;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        let decrypted = self.into_inner().decrypt(key, path)?;
        Ok(EncryptedField::new(decrypted))
    }
}

/// Convert SyncCipher (from API) to Cipher<Encrypted>
impl From<SyncCipher> for Cipher<Encrypted> {
    fn from(sync: SyncCipher) -> Self {
        use super::{
            CipherAttachment, CipherCard, CipherData, CipherField, CipherIdentity, CipherLogin,
            CipherLoginFido2Credential, CipherLoginUri, CipherPasswordHistory, CipherPermissions,
            CipherSecureNote, CipherSshKey,
        };

        Cipher {
            id: sync.id,
            organization_id: sync.organization_id,
            folder_id: sync.folder_id,
            r#type: sync.r#type,
            name: EncryptedField::new(sync.name),
            notes: EncryptedField::new(sync.notes),
            key: sync.key,
            favorite: sync.favorite,
            edit: sync.edit,
            view_password: sync.view_password,
            organization_use_totp: sync.organization_use_totp,
            creation_date: sync.creation_date,
            revision_date: sync.revision_date,
            deleted_date: sync.deleted_date,
            archived_date: sync.archived_date,
            reprompt: sync.reprompt,
            permissions: sync.permissions.map(|p| CipherPermissions {
                delete: p.delete,
                restore: p.restore,
            }),
            object: sync.object,
            fields: sync
                .fields
                .into_iter()
                .map(|f| CipherField {
                    name: EncryptedField::new(f.name),
                    value: EncryptedField::new(f.value),
                    r#type: f.r#type,
                    linked_id: f.linked_id,
                })
                .collect(),
            password_history: sync
                .password_history
                .into_iter()
                .map(|h| CipherPasswordHistory {
                    password: EncryptedField::new(h.password),
                    last_used_date: h.last_used_date,
                })
                .collect(),
            collection_ids: sync.collection_ids,
            data: sync.data.map(|d| CipherData {
                name: EncryptedField::new(d.name),
                notes: EncryptedField::new(d.notes),
                fields: d
                    .fields
                    .into_iter()
                    .map(|f| CipherField {
                        name: EncryptedField::new(f.name),
                        value: EncryptedField::new(f.value),
                        r#type: f.r#type,
                        linked_id: f.linked_id,
                    })
                    .collect(),
                password_history: d
                    .password_history
                    .into_iter()
                    .map(|h| CipherPasswordHistory {
                        password: EncryptedField::new(h.password),
                        last_used_date: h.last_used_date,
                    })
                    .collect(),
                uri: EncryptedField::new(d.uri),
                uris: d
                    .uris
                    .into_iter()
                    .map(|u| CipherLoginUri {
                        uri: EncryptedField::new(u.uri),
                        r#match: u.r#match,
                        uri_checksum: u.uri_checksum,
                    })
                    .collect(),
                username: EncryptedField::new(d.username),
                password: EncryptedField::new(d.password),
                password_revision_date: d.password_revision_date,
                totp: EncryptedField::new(d.totp),
                autofill_on_page_load: d.autofill_on_page_load,
                fido2_credentials: d
                    .fido2_credentials
                    .into_iter()
                    .map(|c| CipherLoginFido2Credential {
                        credential_id: EncryptedField::new(c.credential_id),
                        key_type: EncryptedField::new(c.key_type),
                        key_algorithm: EncryptedField::new(c.key_algorithm),
                        key_curve: EncryptedField::new(c.key_curve),
                        key_value: EncryptedField::new(c.key_value),
                        rp_id: EncryptedField::new(c.rp_id),
                        rp_name: EncryptedField::new(c.rp_name),
                        counter: EncryptedField::new(c.counter),
                        user_handle: EncryptedField::new(c.user_handle),
                        user_name: EncryptedField::new(c.user_name),
                        user_display_name: EncryptedField::new(c.user_display_name),
                        discoverable: EncryptedField::new(c.discoverable),
                        creation_date: EncryptedField::new(c.creation_date),
                    })
                    .collect(),
                r#type: d.r#type,
                cardholder_name: EncryptedField::new(d.cardholder_name),
                brand: EncryptedField::new(d.brand),
                number: EncryptedField::new(d.number),
                exp_month: EncryptedField::new(d.exp_month),
                exp_year: EncryptedField::new(d.exp_year),
                code: EncryptedField::new(d.code),
                title: EncryptedField::new(d.title),
                first_name: EncryptedField::new(d.first_name),
                middle_name: EncryptedField::new(d.middle_name),
                last_name: EncryptedField::new(d.last_name),
                address1: EncryptedField::new(d.address1),
                address2: EncryptedField::new(d.address2),
                address3: EncryptedField::new(d.address3),
                city: EncryptedField::new(d.city),
                state: EncryptedField::new(d.state),
                postal_code: EncryptedField::new(d.postal_code),
                country: EncryptedField::new(d.country),
                company: EncryptedField::new(d.company),
                email: EncryptedField::new(d.email),
                phone: EncryptedField::new(d.phone),
                ssn: EncryptedField::new(d.ssn),
                passport_number: EncryptedField::new(d.passport_number),
                license_number: EncryptedField::new(d.license_number),
                private_key: EncryptedField::new(d.private_key),
                public_key: EncryptedField::new(d.public_key),
                key_fingerprint: EncryptedField::new(d.key_fingerprint),
            }),
            login: sync.login.map(|l| CipherLogin {
                uri: EncryptedField::new(l.uri),
                uris: l
                    .uris
                    .into_iter()
                    .map(|u| CipherLoginUri {
                        uri: EncryptedField::new(u.uri),
                        r#match: u.r#match,
                        uri_checksum: u.uri_checksum,
                    })
                    .collect(),
                username: EncryptedField::new(l.username),
                password: EncryptedField::new(l.password),
                password_revision_date: l.password_revision_date,
                totp: EncryptedField::new(l.totp),
                autofill_on_page_load: l.autofill_on_page_load,
                fido2_credentials: l
                    .fido2_credentials
                    .into_iter()
                    .map(|c| CipherLoginFido2Credential {
                        credential_id: EncryptedField::new(c.credential_id),
                        key_type: EncryptedField::new(c.key_type),
                        key_algorithm: EncryptedField::new(c.key_algorithm),
                        key_curve: EncryptedField::new(c.key_curve),
                        key_value: EncryptedField::new(c.key_value),
                        rp_id: EncryptedField::new(c.rp_id),
                        rp_name: EncryptedField::new(c.rp_name),
                        counter: EncryptedField::new(c.counter),
                        user_handle: EncryptedField::new(c.user_handle),
                        user_name: EncryptedField::new(c.user_name),
                        user_display_name: EncryptedField::new(c.user_display_name),
                        discoverable: EncryptedField::new(c.discoverable),
                        creation_date: EncryptedField::new(c.creation_date),
                    })
                    .collect(),
            }),
            secure_note: sync
                .secure_note
                .map(|n| CipherSecureNote { r#type: n.r#type }),
            card: sync.card.map(|c| CipherCard {
                cardholder_name: EncryptedField::new(c.cardholder_name),
                brand: EncryptedField::new(c.brand),
                number: EncryptedField::new(c.number),
                exp_month: EncryptedField::new(c.exp_month),
                exp_year: EncryptedField::new(c.exp_year),
                code: EncryptedField::new(c.code),
            }),
            identity: sync.identity.map(|i| CipherIdentity {
                title: EncryptedField::new(i.title),
                first_name: EncryptedField::new(i.first_name),
                middle_name: EncryptedField::new(i.middle_name),
                last_name: EncryptedField::new(i.last_name),
                address1: EncryptedField::new(i.address1),
                address2: EncryptedField::new(i.address2),
                address3: EncryptedField::new(i.address3),
                city: EncryptedField::new(i.city),
                state: EncryptedField::new(i.state),
                postal_code: EncryptedField::new(i.postal_code),
                country: EncryptedField::new(i.country),
                company: EncryptedField::new(i.company),
                email: EncryptedField::new(i.email),
                phone: EncryptedField::new(i.phone),
                ssn: EncryptedField::new(i.ssn),
                username: EncryptedField::new(i.username),
                passport_number: EncryptedField::new(i.passport_number),
                license_number: EncryptedField::new(i.license_number),
            }),
            ssh_key: sync.ssh_key.map(|k| CipherSshKey {
                private_key: EncryptedField::new(k.private_key),
                public_key: EncryptedField::new(k.public_key),
                key_fingerprint: EncryptedField::new(k.key_fingerprint),
            }),
            attachments: sync
                .attachments
                .into_iter()
                .map(|a| CipherAttachment {
                    id: a.id,
                    key: a.key,
                    file_name: EncryptedField::new(a.file_name),
                    size: a.size,
                    size_name: a.size_name,
                    url: a.url,
                    object: a.object,
                })
                .collect(),
            _state: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_field_type_aliases() {
        // Just compile-time test to ensure type aliases work
        let _encrypted: EncryptedField<Encrypted, String> =
            EncryptedField::new(Some("test".to_string()));
        let _decrypted: EncryptedField<Decrypted, String> =
            EncryptedField::new(Some("test".to_string()));
    }
}
