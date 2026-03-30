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
    organization_id: detail.organizationId ?? null,
    folder_id: detail.folderId ?? null,
    type: detail.type ?? null,
    name: detail.name,
    notes: detail.notes,
    key: detail.key ?? null,
    favorite: detail.favorite ?? null,
    edit: detail.edit ?? null,
    view_password: detail.viewPassword ?? null,
    organization_use_totp: detail.organizationUseTotp ?? null,
    creation_date: detail.creationDate ?? null,
    revision_date: detail.revisionDate ?? null,
    deleted_date: detail.deletedDate ?? null,
    archived_date: detail.archivedDate ?? null,
    reprompt: detail.reprompt ?? null,
    permissions: detail.permissions
      ? {
          delete: detail.permissions.delete ?? null,
          restore: detail.permissions.restore ?? null,
        }
      : null,
    object: detail.object ?? null,
    fields: (detail.fields ?? []).map((f) => ({
      name: f.name,
      value: f.value,
      type: f.type ?? null,
      linked_id: f.linkedId ?? null,
    })),
    password_history: (detail.passwordHistory ?? []).map((h) => ({
      password: h.password,
      last_used_date: h.lastUsedDate ?? null,
    })),
    collection_ids: detail.collectionIds ?? [],
    data: detail.data
      ? {
          name: detail.data.name,
          notes: detail.data.notes ?? null,
          fields: (detail.data.fields ?? []).map((f) => ({
            name: f.name,
            value: f.value,
            type: f.type ?? null,
            linked_id: f.linkedId ?? null,
          })),
          password_history: (detail.data.passwordHistory ?? []).map((h) => ({
            password: h.password,
            last_used_date: h.lastUsedDate ?? null,
          })),
          uri: detail.data.uri ?? null,
          uris: (detail.data.uris ?? []).map((u) => ({
            uri: u.uri,
            match: u.match ?? null,
            uri_checksum: u.uriChecksum ?? null,
          })),
          username: detail.data.username ?? null,
          password: detail.data.password ?? null,
          password_revision_date: detail.data.passwordRevisionDate ?? null,
          totp: detail.data.totp ?? null,
          autofill_on_page_load: detail.data.autofillOnPageLoad ?? null,
          fido2_credentials: (detail.data.fido2Credentials ?? []).map((c) => ({
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
          type: detail.data.type ?? null,
          cardholder_name: detail.data.cardholderName ?? null,
          brand: detail.data.brand ?? null,
          number: detail.data.number ?? null,
          exp_month: detail.data.expMonth ?? null,
          exp_year: detail.data.expYear ?? null,
          code: detail.data.code ?? null,
          title: detail.data.title ?? null,
          first_name: detail.data.firstName ?? null,
          middle_name: detail.data.middleName ?? null,
          last_name: detail.data.lastName ?? null,
          address1: detail.data.address1 ?? null,
          address2: detail.data.address2 ?? null,
          address3: detail.data.address3 ?? null,
          city: detail.data.city ?? null,
          state: detail.data.state ?? null,
          postal_code: detail.data.postalCode ?? null,
          country: detail.data.country ?? null,
          company: detail.data.company ?? null,
          email: detail.data.email ?? null,
          phone: detail.data.phone ?? null,
          ssn: detail.data.ssn ?? null,
          passport_number: detail.data.passportNumber ?? null,
          license_number: detail.data.licenseNumber ?? null,
          private_key: detail.data.privateKey ?? null,
          public_key: detail.data.publicKey ?? null,
          key_fingerprint: detail.data.keyFingerprint ?? null,
        }
      : null,
    login: detail.login
      ? {
          uri: detail.login.uri ?? null,
          uris: (detail.login.uris ?? []).map((u) => ({
            uri: u.uri,
            match: u.match ?? null,
            uri_checksum: u.uriChecksum ?? null,
          })),
          username: detail.login.username ?? null,
          password: detail.login.password ?? null,
          password_revision_date: detail.login.passwordRevisionDate ?? null,
          totp: detail.login.totp ?? null,
          autofill_on_page_load: detail.login.autofillOnPageLoad ?? null,
          fido2_credentials: (detail.login.fido2Credentials ?? []).map((c) => ({
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
    secure_note: detail.secureNote
      ? { type: detail.secureNote.type ?? null }
      : null,
    card: detail.card
      ? {
          cardholder_name: detail.card.cardholderName ?? null,
          brand: detail.card.brand ?? null,
          number: detail.card.number ?? null,
          exp_month: detail.card.expMonth ?? null,
          exp_year: detail.card.expYear ?? null,
          code: detail.card.code ?? null,
        }
      : null,
    identity: detail.identity
      ? {
          title: detail.identity.title ?? null,
          first_name: detail.identity.firstName ?? null,
          middle_name: detail.identity.middleName ?? null,
          last_name: detail.identity.lastName ?? null,
          address1: detail.identity.address1 ?? null,
          address2: detail.identity.address2 ?? null,
          address3: detail.identity.address3 ?? null,
          city: detail.identity.city ?? null,
          state: detail.identity.state ?? null,
          postal_code: detail.identity.postalCode ?? null,
          country: detail.identity.country ?? null,
          company: detail.identity.company ?? null,
          email: detail.identity.email ?? null,
          phone: detail.identity.phone ?? null,
          ssn: detail.identity.ssn ?? null,
          username: detail.identity.username ?? null,
          passport_number: detail.identity.passportNumber ?? null,
          license_number: detail.identity.licenseNumber ?? null,
        }
      : null,
    ssh_key: detail.sshKey
      ? {
          private_key: detail.sshKey.privateKey ?? null,
          public_key: detail.sshKey.publicKey ?? null,
          key_fingerprint: detail.sshKey.keyFingerprint ?? null,
        }
      : null,
    attachments: (detail.attachments ?? []).map((a) => ({
      id: a.id,
      key: a.key ?? null,
      file_name: a.fileName ?? null,
      size: a.size ?? null,
      size_name: a.sizeName ?? null,
      url: a.url ?? null,
      object: a.object ?? null,
    })),
  };
}
