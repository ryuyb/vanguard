use crate::application::dto::sync::{
    SyncAttachment, SyncCipher, SyncCipherCard, SyncCipherData, SyncCipherField,
    SyncCipherIdentity, SyncCipherLogin, SyncCipherLoginFido2Credential, SyncCipherLoginUri,
    SyncCipherPasswordHistory, SyncCipherPermissions, SyncCipherSecureNote, SyncCipherSshKey,
    SyncCollection, SyncDomains, SyncFolder, SyncKdfParams, SyncMasterPasswordUnlock, SyncPolicy,
    SyncProfile, SyncSend, SyncSendFile, SyncSendText, SyncUserDecryption, SyncVaultPayload,
};

use super::models::{
    SendRequestModel, SyncAttachment as RemoteSyncAttachment, SyncCipher as RemoteSyncCipher,
    SyncCipherCard as RemoteSyncCipherCard, SyncCipherData as RemoteSyncCipherData,
    SyncCipherField as RemoteSyncCipherField, SyncCipherIdentity as RemoteSyncCipherIdentity,
    SyncCipherLogin as RemoteSyncCipherLogin,
    SyncCipherLoginFido2Credential as RemoteSyncCipherLoginFido2Credential,
    SyncCipherLoginUri as RemoteSyncCipherLoginUri,
    SyncCipherPasswordHistory as RemoteSyncCipherPasswordHistory,
    SyncCipherPermissions as RemoteSyncCipherPermissions,
    SyncCipherSecureNote as RemoteSyncCipherSecureNote, SyncCipherSshKey as RemoteSyncCipherSshKey,
    SyncFolder as RemoteSyncFolder, SyncGlobalEquivalentDomainEntry, SyncResponse,
    SyncSend as RemoteSyncSend, SyncSendFile as RemoteSyncSendFile,
    SyncSendText as RemoteSyncSendText,
};

pub fn map_sync_response(response: SyncResponse) -> SyncVaultPayload {
    SyncVaultPayload {
        profile: SyncProfile {
            id: response.profile.id,
            name: response.profile.name,
            email: response.profile.email,
            object: response.profile.object,
        },
        folders: response.folders.into_iter().map(map_sync_folder).collect(),
        collections: response
            .collections
            .into_iter()
            .map(|collection| SyncCollection {
                id: collection.id,
                organization_id: collection.organization_id,
                name: collection.name,
                revision_date: collection.revision_date,
                object: collection.object,
            })
            .collect(),
        policies: response
            .policies
            .into_iter()
            .map(|policy| SyncPolicy {
                id: policy.id,
                organization_id: policy.organization_id,
                r#type: policy.r#type,
                enabled: policy.enabled,
                object: policy.object,
            })
            .collect(),
        ciphers: response.ciphers.into_iter().map(map_sync_cipher).collect(),
        domains: response.domains.map(|domains| {
            let mut global_equivalent_domains = Vec::new();
            let mut excluded_from_globals = Vec::new();

            for entry in domains.global_equivalent_domains {
                match entry {
                    SyncGlobalEquivalentDomainEntry::Legacy(domains) => {
                        global_equivalent_domains.push(domains);
                    }
                    SyncGlobalEquivalentDomainEntry::Detailed(global) => {
                        if global.excluded.unwrap_or(false) {
                            if let Some(global_type) = global.r#type {
                                excluded_from_globals.push(global_type);
                            }
                        }
                        global_equivalent_domains.push(global.domains);
                    }
                }
            }

            let excluded_global_equivalent_domains =
                if domains.excluded_global_equivalent_domains.is_empty() {
                    excluded_from_globals
                } else {
                    domains.excluded_global_equivalent_domains
                };

            SyncDomains {
                equivalent_domains: domains.equivalent_domains,
                global_equivalent_domains,
                excluded_global_equivalent_domains,
            }
        }),
        sends: response.sends.into_iter().map(map_sync_send).collect(),
        user_decryption: response
            .user_decryption
            .map(|decryption| SyncUserDecryption {
                master_password_unlock: decryption.master_password_unlock.map(|unlock| {
                    SyncMasterPasswordUnlock {
                        kdf: unlock.kdf.map(|kdf| SyncKdfParams {
                            kdf_type: kdf.kdf_type,
                            iterations: kdf.iterations,
                            memory: kdf.memory,
                            parallelism: kdf.parallelism,
                        }),
                        master_key_encrypted_user_key: unlock.master_key_encrypted_user_key,
                        master_key_wrapped_user_key: unlock.master_key_wrapped_user_key,
                        salt: unlock.salt,
                    }
                }),
            }),
    }
}

pub fn map_sync_folder(folder: RemoteSyncFolder) -> SyncFolder {
    SyncFolder {
        id: folder.id,
        name: folder.name,
        revision_date: folder.revision_date,
        object: folder.object,
    }
}

pub fn map_sync_cipher(cipher: RemoteSyncCipher) -> SyncCipher {
    SyncCipher {
        id: cipher.id,
        organization_id: cipher.organization_id,
        folder_id: cipher.folder_id,
        r#type: cipher.r#type,
        name: cipher.name,
        notes: cipher.notes,
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
        permissions: cipher.permissions.map(map_sync_cipher_permissions),
        object: cipher.object,
        fields: cipher
            .fields
            .into_iter()
            .map(map_sync_cipher_field)
            .collect(),
        password_history: cipher
            .password_history
            .into_iter()
            .map(map_sync_cipher_password_history)
            .collect(),
        collection_ids: cipher.collection_ids,
        data: cipher.data.map(map_sync_cipher_data),
        login: cipher.login.map(map_sync_cipher_login),
        secure_note: cipher.secure_note.map(map_sync_cipher_secure_note),
        card: cipher.card.map(map_sync_cipher_card),
        identity: cipher.identity.map(map_sync_cipher_identity),
        ssh_key: cipher.ssh_key.map(map_sync_cipher_ssh_key),
        attachments: cipher
            .attachments
            .into_iter()
            .map(map_sync_attachment)
            .collect(),
    }
}

fn map_sync_attachment(attachment: RemoteSyncAttachment) -> SyncAttachment {
    SyncAttachment {
        id: attachment.id,
        key: attachment.key,
        file_name: attachment.file_name,
        size: attachment.size,
        size_name: attachment.size_name,
        url: attachment.url,
        object: attachment.object,
    }
}

fn map_sync_cipher_permissions(permissions: RemoteSyncCipherPermissions) -> SyncCipherPermissions {
    SyncCipherPermissions {
        delete: permissions.delete,
        restore: permissions.restore,
    }
}

fn map_sync_cipher_field(field: RemoteSyncCipherField) -> SyncCipherField {
    SyncCipherField {
        name: field.name,
        value: field.value,
        r#type: field.r#type,
        linked_id: field.linked_id,
    }
}

fn map_sync_cipher_password_history(
    history: RemoteSyncCipherPasswordHistory,
) -> SyncCipherPasswordHistory {
    SyncCipherPasswordHistory {
        password: history.password,
        last_used_date: history.last_used_date,
    }
}

fn map_sync_cipher_data(data: RemoteSyncCipherData) -> SyncCipherData {
    SyncCipherData {
        name: data.name,
        notes: data.notes,
        fields: data.fields.into_iter().map(map_sync_cipher_field).collect(),
        password_history: data
            .password_history
            .into_iter()
            .map(map_sync_cipher_password_history)
            .collect(),
        uri: data.uri,
        uris: data
            .uris
            .into_iter()
            .map(map_sync_cipher_login_uri)
            .collect(),
        username: data.username,
        password: data.password,
        password_revision_date: data.password_revision_date,
        totp: data.totp,
        autofill_on_page_load: data.autofill_on_page_load,
        fido2_credentials: data
            .fido2_credentials
            .into_iter()
            .map(map_sync_cipher_login_fido2_credential)
            .collect(),
        r#type: data.r#type,
        cardholder_name: data.cardholder_name,
        brand: data.brand,
        number: data.number,
        exp_month: data.exp_month,
        exp_year: data.exp_year,
        code: data.code,
        title: data.title,
        first_name: data.first_name,
        middle_name: data.middle_name,
        last_name: data.last_name,
        address1: data.address1,
        address2: data.address2,
        address3: data.address3,
        city: data.city,
        state: data.state,
        postal_code: data.postal_code,
        country: data.country,
        company: data.company,
        email: data.email,
        phone: data.phone,
        ssn: data.ssn,
        passport_number: data.passport_number,
        license_number: data.license_number,
        private_key: data.private_key,
        public_key: data.public_key,
        key_fingerprint: data.key_fingerprint,
    }
}

fn map_sync_cipher_login(login: RemoteSyncCipherLogin) -> SyncCipherLogin {
    SyncCipherLogin {
        uri: login.uri,
        uris: login
            .uris
            .into_iter()
            .map(map_sync_cipher_login_uri)
            .collect(),
        username: login.username,
        password: login.password,
        password_revision_date: login.password_revision_date,
        totp: login.totp,
        autofill_on_page_load: login.autofill_on_page_load,
        fido2_credentials: login
            .fido2_credentials
            .into_iter()
            .map(map_sync_cipher_login_fido2_credential)
            .collect(),
    }
}

fn map_sync_cipher_login_uri(uri: RemoteSyncCipherLoginUri) -> SyncCipherLoginUri {
    SyncCipherLoginUri {
        uri: uri.uri,
        r#match: uri.r#match,
        uri_checksum: uri.uri_checksum,
    }
}

fn map_sync_cipher_login_fido2_credential(
    credential: RemoteSyncCipherLoginFido2Credential,
) -> SyncCipherLoginFido2Credential {
    SyncCipherLoginFido2Credential {
        credential_id: credential.credential_id,
        key_type: credential.key_type,
        key_algorithm: credential.key_algorithm,
        key_curve: credential.key_curve,
        key_value: credential.key_value,
        rp_id: credential.rp_id,
        rp_name: credential.rp_name,
        counter: credential.counter,
        user_handle: credential.user_handle,
        user_name: credential.user_name,
        user_display_name: credential.user_display_name,
        discoverable: credential.discoverable,
        creation_date: credential.creation_date,
    }
}

fn map_sync_cipher_secure_note(note: RemoteSyncCipherSecureNote) -> SyncCipherSecureNote {
    SyncCipherSecureNote {
        r#type: note.r#type,
    }
}

fn map_sync_cipher_card(card: RemoteSyncCipherCard) -> SyncCipherCard {
    SyncCipherCard {
        cardholder_name: card.cardholder_name,
        brand: card.brand,
        number: card.number,
        exp_month: card.exp_month,
        exp_year: card.exp_year,
        code: card.code,
    }
}

fn map_sync_cipher_identity(identity: RemoteSyncCipherIdentity) -> SyncCipherIdentity {
    SyncCipherIdentity {
        title: identity.title,
        first_name: identity.first_name,
        middle_name: identity.middle_name,
        last_name: identity.last_name,
        address1: identity.address1,
        address2: identity.address2,
        address3: identity.address3,
        city: identity.city,
        state: identity.state,
        postal_code: identity.postal_code,
        country: identity.country,
        company: identity.company,
        email: identity.email,
        phone: identity.phone,
        ssn: identity.ssn,
        username: identity.username,
        passport_number: identity.passport_number,
        license_number: identity.license_number,
    }
}

fn map_sync_cipher_ssh_key(ssh_key: RemoteSyncCipherSshKey) -> SyncCipherSshKey {
    SyncCipherSshKey {
        private_key: ssh_key.private_key,
        public_key: ssh_key.public_key,
        key_fingerprint: ssh_key.key_fingerprint,
    }
}

pub fn map_sync_send(send: RemoteSyncSend) -> SyncSend {
    SyncSend {
        id: send.id,
        r#type: send.r#type,
        name: send.name,
        revision_date: send.revision_date,
        deletion_date: send.deletion_date,
        object: send.object,
        access_id: send.access_id,
        notes: send.notes,
        key: send.key,
        password: send.password,
        text: send.text.map(|t| SyncSendText {
            text: t.text,
            hidden: t.hidden,
        }),
        file: send.file.map(|f| SyncSendFile {
            id: f.id,
            file_name: f.file_name,
            size: f.size,
            size_name: f.size_name,
        }),
        max_access_count: send.max_access_count,
        access_count: send.access_count,
        disabled: send.disabled,
        hide_email: send.hide_email,
        expiration_date: send.expiration_date,
        emails: send.emails,
        auth_type: send.auth_type,
    }
}

pub fn map_send_to_request_model(send: &SyncSend) -> SendRequestModel {
    SendRequestModel {
        r#type: send.r#type,
        auth_type: send.auth_type,
        file_length: None,
        name: send.name.clone(),
        notes: send.notes.clone(),
        key: send.key.clone().unwrap_or_default(),
        max_access_count: send.max_access_count,
        expiration_date: send.expiration_date.clone(),
        deletion_date: send.deletion_date.clone().unwrap_or_default(),
        file: send.file.as_ref().map(|f| RemoteSyncSendFile {
            id: f.id.clone(),
            file_name: f.file_name.clone(),
            size: f.size.clone(),
            size_name: f.size_name.clone(),
        }),
        text: send.text.as_ref().map(|t| RemoteSyncSendText {
            text: t.text.clone(),
            hidden: t.hidden,
        }),
        password: send.password.clone(),
        emails: send.emails.clone(),
        disabled: send.disabled.unwrap_or(false),
        hide_email: send.hide_email,
    }
}
pub fn map_cipher_to_remote(cipher: &SyncCipher) -> RemoteSyncCipher {
    RemoteSyncCipher {
        id: cipher.id.clone(),
        organization_id: cipher.organization_id.clone(),
        folder_id: cipher.folder_id.clone(),
        r#type: cipher.r#type,
        name: cipher.name.clone(),
        notes: cipher.notes.clone(),
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
        permissions: cipher
            .permissions
            .as_ref()
            .map(|p| RemoteSyncCipherPermissions {
                delete: p.delete,
                restore: p.restore,
            }),
        object: cipher.object.clone(),
        fields: cipher
            .fields
            .iter()
            .map(|f| RemoteSyncCipherField {
                name: f.name.clone(),
                value: f.value.clone(),
                r#type: f.r#type,
                linked_id: f.linked_id,
            })
            .collect(),
        password_history: cipher
            .password_history
            .iter()
            .map(|h| RemoteSyncCipherPasswordHistory {
                password: h.password.clone(),
                last_used_date: h.last_used_date.clone(),
            })
            .collect(),
        collection_ids: cipher.collection_ids.clone(),
        data: cipher.data.as_ref().map(|d| RemoteSyncCipherData {
            name: d.name.clone(),
            notes: d.notes.clone(),
            fields: d
                .fields
                .iter()
                .map(|f| RemoteSyncCipherField {
                    name: f.name.clone(),
                    value: f.value.clone(),
                    r#type: f.r#type,
                    linked_id: f.linked_id,
                })
                .collect(),
            password_history: d
                .password_history
                .iter()
                .map(|h| RemoteSyncCipherPasswordHistory {
                    password: h.password.clone(),
                    last_used_date: h.last_used_date.clone(),
                })
                .collect(),
            uri: d.uri.clone(),
            uris: d
                .uris
                .iter()
                .map(|u| RemoteSyncCipherLoginUri {
                    uri: u.uri.clone(),
                    r#match: u.r#match,
                    uri_checksum: u.uri_checksum.clone(),
                })
                .collect(),
            username: d.username.clone(),
            password: d.password.clone(),
            password_revision_date: d.password_revision_date.clone(),
            totp: d.totp.clone(),
            autofill_on_page_load: d.autofill_on_page_load,
            fido2_credentials: d
                .fido2_credentials
                .iter()
                .map(|c| RemoteSyncCipherLoginFido2Credential {
                    credential_id: c.credential_id.clone(),
                    key_type: c.key_type.clone(),
                    key_algorithm: c.key_algorithm.clone(),
                    key_curve: c.key_curve.clone(),
                    key_value: c.key_value.clone(),
                    rp_id: c.rp_id.clone(),
                    rp_name: c.rp_name.clone(),
                    counter: c.counter.clone(),
                    user_handle: c.user_handle.clone(),
                    user_name: c.user_name.clone(),
                    user_display_name: c.user_display_name.clone(),
                    discoverable: c.discoverable.clone(),
                    creation_date: c.creation_date.clone(),
                })
                .collect(),
            r#type: d.r#type,
            cardholder_name: d.cardholder_name.clone(),
            brand: d.brand.clone(),
            number: d.number.clone(),
            exp_month: d.exp_month.clone(),
            exp_year: d.exp_year.clone(),
            code: d.code.clone(),
            title: d.title.clone(),
            first_name: d.first_name.clone(),
            middle_name: d.middle_name.clone(),
            last_name: d.last_name.clone(),
            address1: d.address1.clone(),
            address2: d.address2.clone(),
            address3: d.address3.clone(),
            city: d.city.clone(),
            state: d.state.clone(),
            postal_code: d.postal_code.clone(),
            country: d.country.clone(),
            company: d.company.clone(),
            email: d.email.clone(),
            phone: d.phone.clone(),
            ssn: d.ssn.clone(),
            passport_number: d.passport_number.clone(),
            license_number: d.license_number.clone(),
            private_key: d.private_key.clone(),
            public_key: d.public_key.clone(),
            key_fingerprint: d.key_fingerprint.clone(),
        }),
        login: cipher.login.as_ref().map(|l| RemoteSyncCipherLogin {
            uri: l.uri.clone(),
            uris: l
                .uris
                .iter()
                .map(|u| RemoteSyncCipherLoginUri {
                    uri: u.uri.clone(),
                    r#match: u.r#match,
                    uri_checksum: u.uri_checksum.clone(),
                })
                .collect(),
            username: l.username.clone(),
            password: l.password.clone(),
            password_revision_date: l.password_revision_date.clone(),
            totp: l.totp.clone(),
            autofill_on_page_load: l.autofill_on_page_load,
            fido2_credentials: l
                .fido2_credentials
                .iter()
                .map(|c| RemoteSyncCipherLoginFido2Credential {
                    credential_id: c.credential_id.clone(),
                    key_type: c.key_type.clone(),
                    key_algorithm: c.key_algorithm.clone(),
                    key_curve: c.key_curve.clone(),
                    key_value: c.key_value.clone(),
                    rp_id: c.rp_id.clone(),
                    rp_name: c.rp_name.clone(),
                    counter: c.counter.clone(),
                    user_handle: c.user_handle.clone(),
                    user_name: c.user_name.clone(),
                    user_display_name: c.user_display_name.clone(),
                    discoverable: c.discoverable.clone(),
                    creation_date: c.creation_date.clone(),
                })
                .collect(),
        }),
        secure_note: cipher
            .secure_note
            .as_ref()
            .map(|n| RemoteSyncCipherSecureNote { r#type: n.r#type }),
        card: cipher.card.as_ref().map(|c| RemoteSyncCipherCard {
            cardholder_name: c.cardholder_name.clone(),
            brand: c.brand.clone(),
            number: c.number.clone(),
            exp_month: c.exp_month.clone(),
            exp_year: c.exp_year.clone(),
            code: c.code.clone(),
        }),
        identity: cipher.identity.as_ref().map(|i| RemoteSyncCipherIdentity {
            title: i.title.clone(),
            first_name: i.first_name.clone(),
            middle_name: i.middle_name.clone(),
            last_name: i.last_name.clone(),
            address1: i.address1.clone(),
            address2: i.address2.clone(),
            address3: i.address3.clone(),
            city: i.city.clone(),
            state: i.state.clone(),
            postal_code: i.postal_code.clone(),
            country: i.country.clone(),
            company: i.company.clone(),
            email: i.email.clone(),
            phone: i.phone.clone(),
            ssn: i.ssn.clone(),
            username: i.username.clone(),
            passport_number: i.passport_number.clone(),
            license_number: i.license_number.clone(),
        }),
        ssh_key: cipher.ssh_key.as_ref().map(|s| RemoteSyncCipherSshKey {
            private_key: s.private_key.clone(),
            public_key: s.public_key.clone(),
            key_fingerprint: s.key_fingerprint.clone(),
        }),
        attachments: cipher
            .attachments
            .iter()
            .map(|a| RemoteSyncAttachment {
                id: a.id.clone(),
                key: a.key.clone(),
                file_name: a.file_name.clone(),
                size: a.size.clone(),
                size_name: a.size_name.clone(),
                url: a.url.clone(),
                object: a.object.clone(),
            })
            .collect(),
    }
}
#[cfg(test)]
mod tests {
    use super::map_sync_cipher;
    use crate::infrastructure::vaultwarden::models::{
        SyncAttachment as RemoteSyncAttachment, SyncCipher as RemoteSyncCipher,
        SyncCipherData as RemoteSyncCipherData, SyncCipherField as RemoteSyncCipherField,
        SyncCipherLogin as RemoteSyncCipherLogin, SyncCipherLoginUri as RemoteSyncCipherLoginUri,
        SyncCipherPasswordHistory as RemoteSyncCipherPasswordHistory,
        SyncCipherPermissions as RemoteSyncCipherPermissions,
    };

    #[test]
    fn map_sync_cipher_maps_explicit_fields() {
        let remote = RemoteSyncCipher {
            id: String::from("cipher-1"),
            organization_id: Some(String::from("org-1")),
            folder_id: Some(String::from("folder-1")),
            r#type: Some(1),
            name: Some(String::from("cipher-name")),
            notes: Some(String::from("cipher-notes")),
            key: Some(String::from("cipher-key")),
            favorite: Some(true),
            edit: Some(true),
            view_password: Some(false),
            organization_use_totp: Some(false),
            creation_date: Some(String::from("2026-03-01T00:00:00.000000Z")),
            revision_date: Some(String::from("2026-03-01T01:00:00.000000Z")),
            deleted_date: None,
            archived_date: None,
            reprompt: Some(1),
            permissions: Some(RemoteSyncCipherPermissions {
                delete: Some(true),
                restore: Some(false),
            }),
            object: Some(String::from("cipher")),
            fields: vec![RemoteSyncCipherField {
                name: Some(String::from("field-name")),
                value: Some(String::from("field-value")),
                r#type: Some(0),
                linked_id: Some(1),
            }],
            password_history: vec![RemoteSyncCipherPasswordHistory {
                password: Some(String::from("old-password")),
                last_used_date: Some(String::from("2026-02-28T23:59:00.000000Z")),
            }],
            collection_ids: vec![String::from("col-1")],
            data: Some(RemoteSyncCipherData {
                name: Some(String::from("data-name")),
                notes: Some(String::from("data-notes")),
                fields: vec![],
                password_history: vec![],
                uri: Some(String::from("https://example.com")),
                uris: vec![RemoteSyncCipherLoginUri {
                    uri: Some(String::from("https://example.com/login")),
                    r#match: Some(0),
                    uri_checksum: Some(String::from("checksum")),
                }],
                username: Some(String::from("user@example.com")),
                password: Some(String::from("encrypted-password")),
                password_revision_date: Some(String::from("2026-02-28T23:59:00.000000Z")),
                totp: Some(String::from("encrypted-totp")),
                autofill_on_page_load: Some(false),
                fido2_credentials: vec![],
                r#type: Some(0),
                cardholder_name: None,
                brand: None,
                number: None,
                exp_month: None,
                exp_year: None,
                code: None,
                title: None,
                first_name: None,
                middle_name: None,
                last_name: None,
                address1: None,
                address2: None,
                address3: None,
                city: None,
                state: None,
                postal_code: None,
                country: None,
                company: None,
                email: None,
                phone: None,
                ssn: None,
                passport_number: None,
                license_number: None,
                private_key: None,
                public_key: None,
                key_fingerprint: None,
            }),
            login: Some(RemoteSyncCipherLogin {
                uri: Some(String::from("https://example.com")),
                uris: vec![RemoteSyncCipherLoginUri {
                    uri: Some(String::from("https://example.com/login")),
                    r#match: Some(0),
                    uri_checksum: None,
                }],
                username: Some(String::from("user@example.com")),
                password: Some(String::from("encrypted-password")),
                password_revision_date: None,
                totp: None,
                autofill_on_page_load: Some(true),
                fido2_credentials: vec![],
            }),
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: vec![RemoteSyncAttachment {
                id: String::from("att-1"),
                key: Some(String::from("att-key")),
                file_name: Some(String::from("a.txt")),
                size: Some(String::from("12")),
                size_name: Some(String::from("12 B")),
                url: Some(String::from("https://example.invalid/attachment")),
                object: Some(String::from("attachment")),
            }],
        };

        let mapped = map_sync_cipher(remote);

        assert_eq!(mapped.id, "cipher-1");
        assert_eq!(mapped.notes.as_deref(), Some("cipher-notes"));
        assert_eq!(mapped.key.as_deref(), Some("cipher-key"));
        assert_eq!(mapped.reprompt, Some(1));
        assert_eq!(mapped.collection_ids, vec![String::from("col-1")]);
        assert_eq!(mapped.attachments.len(), 1);
        assert_eq!(mapped.attachments[0].key.as_deref(), Some("att-key"));
        assert_eq!(mapped.attachments[0].size_name.as_deref(), Some("12 B"));
        assert_eq!(
            mapped
                .data
                .as_ref()
                .and_then(|value| value.username.as_deref()),
            Some("user@example.com")
        );
        assert_eq!(
            mapped
                .login
                .as_ref()
                .and_then(|value| value.autofill_on_page_load),
            Some(true)
        );
        assert_eq!(
            mapped.permissions.as_ref().and_then(|value| value.delete),
            Some(true)
        );
    }
}
