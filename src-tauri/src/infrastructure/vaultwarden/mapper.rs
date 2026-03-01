use crate::application::dto::sync::{
    SyncAttachment, SyncCipher, SyncCollection, SyncDomains, SyncFolder, SyncKdfParams,
    SyncMasterPasswordUnlock, SyncPolicy, SyncProfile, SyncSend, SyncUserDecryption,
    SyncVaultPayload,
};

use super::models::{
    SyncCipher as RemoteSyncCipher, SyncFolder as RemoteSyncFolder,
    SyncGlobalEquivalentDomainEntry, SyncResponse, SyncSend as RemoteSyncSend,
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
        revision_date: cipher.revision_date,
        deleted_date: cipher.deleted_date,
        object: cipher.object,
        attachments: cipher
            .attachments
            .into_iter()
            .map(|attachment| SyncAttachment {
                id: attachment.id,
                file_name: attachment.file_name,
                size: attachment.size,
                url: attachment.url,
                object: attachment.object,
            })
            .collect(),
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
    }
}
