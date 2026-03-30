use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::application::dto::sync::{SyncSend, SyncSendFile, SyncSendText};
use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::application::send_encryption;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SendItemDto {
    pub id: String,
    pub r#type: Option<i32>,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub disabled: Option<bool>,
    pub hide_email: Option<bool>,
    pub expiration_date: Option<String>,
    pub deletion_date: Option<String>,
    pub access_count: Option<i32>,
    pub max_access_count: Option<i32>,
    pub has_password: bool,
    pub revision_date: Option<String>,
    pub text: Option<SyncSendText>,
    pub access_id: Option<String>,
    pub key: Option<String>,
    pub url_key: Option<String>,
    pub file_name: Option<String>,
    pub size_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SendDetailDto {
    pub id: String,
    pub r#type: Option<i32>,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub text: Option<SyncSendText>,
    pub file: Option<SyncSendFile>,
    pub disabled: Option<bool>,
    pub hide_email: Option<bool>,
    pub expiration_date: Option<String>,
    pub deletion_date: Option<String>,
    pub access_count: Option<i32>,
    pub max_access_count: Option<i32>,
    pub has_password: bool,
    pub access_id: Option<String>,
    pub key: Option<String>,
    pub emails: Option<String>,
    pub auth_type: Option<i32>,
    pub revision_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateSendRequestDto {
    pub send: SyncSend,
    pub file_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSendRequestDto {
    pub send_id: String,
    pub send: SyncSend,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSendRequestDto {
    pub send_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RemoveSendPasswordRequestDto {
    pub send_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SendMutationResponseDto {
    pub send_id: String,
    pub revision_date: String,
}

pub fn to_send_item_dto(send: SyncSend, user_key: &VaultUserKeyMaterial) -> SendItemDto {
    let url_key = send
        .key
        .as_deref()
        .and_then(|k| send_encryption::decrypt_send_key(k, user_key).ok())
        .map(|raw| URL_SAFE_NO_PAD.encode(raw));

    SendItemDto {
        id: send.id,
        r#type: send.r#type,
        name: send.name,
        notes: send.notes,
        disabled: send.disabled,
        hide_email: send.hide_email,
        expiration_date: send.expiration_date,
        deletion_date: send.deletion_date,
        access_count: send.access_count,
        max_access_count: send.max_access_count,
        has_password: send.password.is_some(),
        revision_date: send.revision_date,
        text: send.text,
        access_id: send.access_id,
        key: send.key,
        url_key,
        file_name: send.file.as_ref().and_then(|f| f.file_name.clone()),
        size_name: send.file.as_ref().and_then(|f| f.size_name.clone()),
    }
}
