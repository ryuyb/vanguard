import type { SyncCipher, VaultCipherDetailDto } from "@/bindings";

/**
 * 将 VaultCipherDetailDto 转换为 SyncCipher
 * 用于编辑 cipher 时的类型转换
 */
export function vaultCipherDetailToSyncCipher(
  detail: VaultCipherDetailDto,
): SyncCipher {
  return {
    id: detail.id,
    organization_id: detail.organizationId,
    folder_id: detail.folderId,
    type: detail.type,
    name: detail.name,
    notes: detail.notes,
    key: detail.key,
    favorite: detail.favorite,
    edit: detail.edit,
    view_password: detail.viewPassword,
    organization_use_totp: detail.organizationUseTotp,
    creation_date: detail.creationDate,
    revision_date: detail.revisionDate,
    deleted_date: detail.deletedDate,
    archived_date: detail.archivedDate,
    reprompt: detail.reprompt,
    permissions: detail.permissions
      ? {
          delete: detail.permissions.delete,
          restore: detail.permissions.restore,
        }
      : null,
    object: detail.object,
    fields: detail.fields.map((f) => ({
      name: f.name,
      value: f.value,
      type: f.type,
      linked_id: f.linkedId,
    })),
    password_history: detail.passwordHistory.map((h) => ({
      password: h.password,
      last_used_date: h.lastUsedDate,
    })),
    collection_ids: detail.collectionIds,
    data: detail.data
      ? {
          name: detail.data.name,
          notes: detail.data.notes,
          fields: detail.data.fields.map((f) => ({
            name: f.name,
            value: f.value,
            type: f.type,
            linked_id: f.linkedId,
          })),
          password_history: detail.data.passwordHistory.map((h) => ({
            password: h.password,
            last_used_date: h.lastUsedDate,
          })),
          uri: detail.data.uri,
          uris: detail.data.uris.map((u) => ({
            uri: u.uri,
            match: u.match,
            uri_checksum: u.uriChecksum,
          })),
          username: detail.data.username,
          password: detail.data.password,
          password_revision_date: detail.data.passwordRevisionDate,
          totp: null,
          autofill_on_page_load: detail.data.autofillOnPageLoad,
          fido2_credentials: detail.data.fido2Credentials.map((c) => ({
            credential_id: c.credentialId,
            key_type: c.keyType,
            key_algorithm: c.keyAlgorithm,
            key_curve: c.keyCurve,
            key_value: c.keyValue,
            rp_id: c.rpId,
            rp_name: c.rpName,
            counter: c.counter,
            user_handle: c.userHandle,
            user_name: c.userName,
            user_display_name: c.userDisplayName,
            discoverable: c.discoverable,
            creation_date: c.creationDate,
          })),
          type: detail.data.type,
          cardholder_name: detail.data.cardholderName,
          brand: detail.data.brand,
          number: detail.data.number,
          exp_month: detail.data.expMonth,
          exp_year: detail.data.expYear,
          code: detail.data.code,
          title: detail.data.title,
          first_name: detail.data.firstName,
          middle_name: detail.data.middleName,
          last_name: detail.data.lastName,
          address1: detail.data.address1,
          address2: detail.data.address2,
          address3: detail.data.address3,
          city: detail.data.city,
          state: detail.data.state,
          postal_code: detail.data.postalCode,
          country: detail.data.country,
          company: detail.data.company,
          email: detail.data.email,
          phone: detail.data.phone,
          ssn: detail.data.ssn,
          passport_number: detail.data.passportNumber,
          license_number: detail.data.licenseNumber,
          private_key: detail.data.privateKey,
          public_key: detail.data.publicKey,
          key_fingerprint: detail.data.keyFingerprint,
        }
      : null,
    login: detail.login
      ? {
          uri: detail.login.uri,
          uris: detail.login.uris.map((u) => ({
            uri: u.uri,
            match: u.match,
            uri_checksum: u.uriChecksum,
          })),
          username: detail.login.username,
          password: detail.login.password,
          password_revision_date: detail.login.passwordRevisionDate,
          totp: null,
          autofill_on_page_load: detail.login.autofillOnPageLoad,
          fido2_credentials: detail.login.fido2Credentials.map((c) => ({
            credential_id: c.credentialId,
            key_type: c.keyType,
            key_algorithm: c.keyAlgorithm,
            key_curve: c.keyCurve,
            key_value: c.keyValue,
            rp_id: c.rpId,
            rp_name: c.rpName,
            counter: c.counter,
            user_handle: c.userHandle,
            user_name: c.userName,
            user_display_name: c.userDisplayName,
            discoverable: c.discoverable,
            creation_date: c.creationDate,
          })),
        }
      : null,
    secure_note: detail.secureNote ? { type: detail.secureNote.type } : null,
    card: detail.card
      ? {
          cardholder_name: detail.card.cardholderName,
          brand: detail.card.brand,
          number: detail.card.number,
          exp_month: detail.card.expMonth,
          exp_year: detail.card.expYear,
          code: detail.card.code,
        }
      : null,
    identity: detail.identity
      ? {
          title: detail.identity.title,
          first_name: detail.identity.firstName,
          middle_name: detail.identity.middleName,
          last_name: detail.identity.lastName,
          address1: detail.identity.address1,
          address2: detail.identity.address2,
          address3: detail.identity.address3,
          city: detail.identity.city,
          state: detail.identity.state,
          postal_code: detail.identity.postalCode,
          country: detail.identity.country,
          company: detail.identity.company,
          email: detail.identity.email,
          phone: detail.identity.phone,
          ssn: detail.identity.ssn,
          username: detail.identity.username,
          passport_number: detail.identity.passportNumber,
          license_number: detail.identity.licenseNumber,
        }
      : null,
    ssh_key: detail.sshKey
      ? {
          private_key: detail.sshKey.privateKey,
          public_key: detail.sshKey.publicKey,
          key_fingerprint: detail.sshKey.keyFingerprint,
        }
      : null,
    attachments: detail.attachments.map((a) => ({
      id: a.id,
      key: a.key,
      file_name: a.fileName,
      size: a.size,
      size_name: a.sizeName,
      url: a.url,
      object: a.object,
    })),
  };
}
