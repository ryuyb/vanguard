use crate::application::dto::sync::{
    SyncAttachment, SyncCipherCard, SyncCipherData, SyncCipherField, SyncCipherIdentity,
    SyncCipherLogin, SyncCipherLoginFido2Credential, SyncCipherLoginUri, SyncCipherPasswordHistory,
    SyncCipherSshKey,
};
use crate::application::dto::vault::{
    VaultAttachmentDetail, VaultCipherCardDetail, VaultCipherDataDetail, VaultCipherFieldDetail,
    VaultCipherIdentityDetail, VaultCipherLoginDetail, VaultCipherLoginFido2CredentialDetail,
    VaultCipherLoginUriDetail, VaultCipherPasswordHistoryDetail, VaultCipherSshKeyDetail,
    VaultUserKeyMaterial,
};
use crate::domain::crypto::Decryptable;
use crate::support::error::AppError;

impl Decryptable for SyncCipherLoginUri {
    type Output = VaultCipherLoginUriDetail;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        Ok(VaultCipherLoginUriDetail {
            uri: self.uri.decrypt(key, &format!("{path}.uri"))?,
            r#match: self.r#match,
            uri_checksum: self.uri_checksum,
        })
    }
}

impl Decryptable for SyncCipherPasswordHistory {
    type Output = VaultCipherPasswordHistoryDetail;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        Ok(VaultCipherPasswordHistoryDetail {
            password: self.password.decrypt(key, &format!("{path}.password"))?,
            last_used_date: self.last_used_date,
        })
    }
}

impl Decryptable for SyncCipherField {
    type Output = VaultCipherFieldDetail;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        Ok(VaultCipherFieldDetail {
            name: self.name.decrypt(key, &format!("{path}.name"))?,
            value: self.value.decrypt(key, &format!("{path}.value"))?,
            r#type: self.r#type,
            linked_id: self.linked_id,
        })
    }
}

impl Decryptable for SyncCipherLoginFido2Credential {
    type Output = VaultCipherLoginFido2CredentialDetail;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        Ok(VaultCipherLoginFido2CredentialDetail {
            credential_id: self
                .credential_id
                .decrypt(key, &format!("{path}.credential_id"))?,
            key_type: self.key_type.decrypt(key, &format!("{path}.key_type"))?,
            key_algorithm: self
                .key_algorithm
                .decrypt(key, &format!("{path}.key_algorithm"))?,
            key_curve: self.key_curve.decrypt(key, &format!("{path}.key_curve"))?,
            key_value: self.key_value.decrypt(key, &format!("{path}.key_value"))?,
            rp_id: self.rp_id.decrypt(key, &format!("{path}.rp_id"))?,
            rp_name: self.rp_name.decrypt(key, &format!("{path}.rp_name"))?,
            counter: self.counter.decrypt(key, &format!("{path}.counter"))?,
            user_handle: self
                .user_handle
                .decrypt(key, &format!("{path}.user_handle"))?,
            user_name: self.user_name.decrypt(key, &format!("{path}.user_name"))?,
            user_display_name: self
                .user_display_name
                .decrypt(key, &format!("{path}.user_display_name"))?,
            discoverable: self
                .discoverable
                .decrypt(key, &format!("{path}.discoverable"))?,
            creation_date: self
                .creation_date
                .decrypt(key, &format!("{path}.creation_date"))?,
        })
    }
}

impl Decryptable for SyncAttachment {
    type Output = VaultAttachmentDetail;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        Ok(VaultAttachmentDetail {
            id: self.id,
            key: self.key,
            file_name: self.file_name.decrypt(key, &format!("{path}.file_name"))?,
            size: self.size,
            size_name: self.size_name,
            url: self.url,
            object: self.object,
        })
    }
}

impl Decryptable for SyncCipherCard {
    type Output = VaultCipherCardDetail;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        Ok(VaultCipherCardDetail {
            cardholder_name: self
                .cardholder_name
                .decrypt(key, &format!("{path}.cardholder_name"))?,
            brand: self.brand.decrypt(key, &format!("{path}.brand"))?,
            number: self.number.decrypt(key, &format!("{path}.number"))?,
            exp_month: self.exp_month.decrypt(key, &format!("{path}.exp_month"))?,
            exp_year: self.exp_year.decrypt(key, &format!("{path}.exp_year"))?,
            code: self.code.decrypt(key, &format!("{path}.code"))?,
        })
    }
}

impl Decryptable for SyncCipherIdentity {
    type Output = VaultCipherIdentityDetail;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        Ok(VaultCipherIdentityDetail {
            title: self.title.decrypt(key, &format!("{path}.title"))?,
            first_name: self
                .first_name
                .decrypt(key, &format!("{path}.first_name"))?,
            middle_name: self
                .middle_name
                .decrypt(key, &format!("{path}.middle_name"))?,
            last_name: self.last_name.decrypt(key, &format!("{path}.last_name"))?,
            address1: self.address1.decrypt(key, &format!("{path}.address1"))?,
            address2: self.address2.decrypt(key, &format!("{path}.address2"))?,
            address3: self.address3.decrypt(key, &format!("{path}.address3"))?,
            city: self.city.decrypt(key, &format!("{path}.city"))?,
            state: self.state.decrypt(key, &format!("{path}.state"))?,
            postal_code: self
                .postal_code
                .decrypt(key, &format!("{path}.postal_code"))?,
            country: self.country.decrypt(key, &format!("{path}.country"))?,
            company: self.company.decrypt(key, &format!("{path}.company"))?,
            email: self.email.decrypt(key, &format!("{path}.email"))?,
            phone: self.phone.decrypt(key, &format!("{path}.phone"))?,
            ssn: self.ssn.decrypt(key, &format!("{path}.ssn"))?,
            username: self.username.decrypt(key, &format!("{path}.username"))?,
            passport_number: self
                .passport_number
                .decrypt(key, &format!("{path}.passport_number"))?,
            license_number: self
                .license_number
                .decrypt(key, &format!("{path}.license_number"))?,
        })
    }
}

impl Decryptable for SyncCipherSshKey {
    type Output = VaultCipherSshKeyDetail;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        Ok(VaultCipherSshKeyDetail {
            private_key: self
                .private_key
                .decrypt(key, &format!("{path}.private_key"))?,
            public_key: self
                .public_key
                .decrypt(key, &format!("{path}.public_key"))?,
            key_fingerprint: self
                .key_fingerprint
                .decrypt(key, &format!("{path}.key_fingerprint"))?,
        })
    }
}

impl Decryptable for SyncCipherLogin {
    type Output = VaultCipherLoginDetail;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        Ok(VaultCipherLoginDetail {
            uri: self.uri.decrypt(key, &format!("{path}.uri"))?,
            uris: self.uris.decrypt(key, &format!("{path}.uris"))?,
            username: self.username.decrypt(key, &format!("{path}.username"))?,
            password: self.password.decrypt(key, &format!("{path}.password"))?,
            password_revision_date: self.password_revision_date,
            totp: self.totp.decrypt(key, &format!("{path}.totp"))?,
            autofill_on_page_load: self.autofill_on_page_load,
            fido2_credentials: self
                .fido2_credentials
                .decrypt(key, &format!("{path}.fido2_credentials"))?,
        })
    }
}

impl Decryptable for SyncCipherData {
    type Output = VaultCipherDataDetail;

    fn decrypt(self, key: &VaultUserKeyMaterial, path: &str) -> Result<Self::Output, AppError> {
        Ok(VaultCipherDataDetail {
            name: self.name.decrypt(key, &format!("{path}.name"))?,
            notes: self.notes.decrypt(key, &format!("{path}.notes"))?,
            fields: self.fields.decrypt(key, &format!("{path}.fields"))?,
            password_history: self
                .password_history
                .decrypt(key, &format!("{path}.password_history"))?,
            uri: self.uri.decrypt(key, &format!("{path}.uri"))?,
            uris: self.uris.decrypt(key, &format!("{path}.uris"))?,
            username: self.username.decrypt(key, &format!("{path}.username"))?,
            password: self.password.decrypt(key, &format!("{path}.password"))?,
            password_revision_date: self.password_revision_date,
            totp: self.totp.decrypt(key, &format!("{path}.totp"))?,
            autofill_on_page_load: self.autofill_on_page_load,
            fido2_credentials: self
                .fido2_credentials
                .decrypt(key, &format!("{path}.fido2_credentials"))?,
            r#type: self.r#type,
            cardholder_name: self
                .cardholder_name
                .decrypt(key, &format!("{path}.cardholder_name"))?,
            brand: self.brand.decrypt(key, &format!("{path}.brand"))?,
            number: self.number.decrypt(key, &format!("{path}.number"))?,
            exp_month: self.exp_month.decrypt(key, &format!("{path}.exp_month"))?,
            exp_year: self.exp_year.decrypt(key, &format!("{path}.exp_year"))?,
            code: self.code.decrypt(key, &format!("{path}.code"))?,
            title: self.title.decrypt(key, &format!("{path}.title"))?,
            first_name: self
                .first_name
                .decrypt(key, &format!("{path}.first_name"))?,
            middle_name: self
                .middle_name
                .decrypt(key, &format!("{path}.middle_name"))?,
            last_name: self.last_name.decrypt(key, &format!("{path}.last_name"))?,
            address1: self.address1.decrypt(key, &format!("{path}.address1"))?,
            address2: self.address2.decrypt(key, &format!("{path}.address2"))?,
            address3: self.address3.decrypt(key, &format!("{path}.address3"))?,
            city: self.city.decrypt(key, &format!("{path}.city"))?,
            state: self.state.decrypt(key, &format!("{path}.state"))?,
            postal_code: self
                .postal_code
                .decrypt(key, &format!("{path}.postal_code"))?,
            country: self.country.decrypt(key, &format!("{path}.country"))?,
            company: self.company.decrypt(key, &format!("{path}.company"))?,
            email: self.email.decrypt(key, &format!("{path}.email"))?,
            phone: self.phone.decrypt(key, &format!("{path}.phone"))?,
            ssn: self.ssn.decrypt(key, &format!("{path}.ssn"))?,
            passport_number: self
                .passport_number
                .decrypt(key, &format!("{path}.passport_number"))?,
            license_number: self
                .license_number
                .decrypt(key, &format!("{path}.license_number"))?,
            private_key: self
                .private_key
                .decrypt(key, &format!("{path}.private_key"))?,
            public_key: self
                .public_key
                .decrypt(key, &format!("{path}.public_key"))?,
            key_fingerprint: self
                .key_fingerprint
                .decrypt(key, &format!("{path}.key_fingerprint"))?,
        })
    }
}
