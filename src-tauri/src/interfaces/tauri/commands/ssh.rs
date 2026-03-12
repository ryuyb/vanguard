use serde::{Deserialize, Serialize};
use ssh_key::PrivateKey;

#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SshKeyResult {
    pub public_key: String,
    pub fingerprint: String,
}

#[tauri::command]
#[specta::specta]
pub async fn extract_ssh_public_key(private_key: String) -> Result<SshKeyResult, String> {
    let private_key = PrivateKey::from_openssh(private_key.trim())
        .map_err(|e| format!("无效的 SSH 私钥格式。请确保使用 OpenSSH 格式（以 '-----BEGIN OPENSSH PRIVATE KEY-----' 开头）: {}", e))?;

    let public_key = private_key.public_key();
    let public_key_str = public_key
        .to_openssh()
        .map_err(|e| format!("Failed to encode public key: {}", e))?;

    let fingerprint = public_key.fingerprint(ssh_key::HashAlg::Sha256);

    Ok(SshKeyResult {
        public_key: public_key_str,
        fingerprint: fingerprint.to_string(),
    })
}
