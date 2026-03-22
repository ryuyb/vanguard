use crate::application::dto::vault::VaultBiometricBundle;
use crate::application::ports::biometric_unlock_port::BiometricUnlockPort;
use crate::infrastructure::security::biometric_store;
use crate::support::result::AppResult;

#[derive(Debug, Default, Clone)]
pub struct KeychainBiometricUnlockPort;

impl BiometricUnlockPort for KeychainBiometricUnlockPort {
    fn is_supported(&self) -> bool {
        biometric_store::is_supported()
    }

    fn save_unlock_bundle(&self, account_id: &str, bundle: &VaultBiometricBundle) -> AppResult<()> {
        biometric_store::save_unlock_bundle(account_id, &to_store_bundle(bundle))
    }

    fn load_unlock_bundle(&self, account_id: &str) -> AppResult<VaultBiometricBundle> {
        biometric_store::load_unlock_bundle(account_id).map(from_store_bundle)
    }

    fn has_unlock_bundle(&self, account_id: &str) -> AppResult<bool> {
        biometric_store::has_unlock_bundle(account_id)
    }

    fn delete_unlock_bundle(&self, account_id: &str) -> AppResult<()> {
        biometric_store::delete_unlock_bundle(account_id)
    }
}

fn to_store_bundle(bundle: &VaultBiometricBundle) -> biometric_store::BiometricUnlockBundle {
    biometric_store::BiometricUnlockBundle::new(
        bundle.account_id.clone(),
        bundle.enc_key_b64.clone(),
        bundle.mac_key_b64.clone(),
        bundle.refresh_token.clone(),
    )
}

fn from_store_bundle(bundle: biometric_store::BiometricUnlockBundle) -> VaultBiometricBundle {
    VaultBiometricBundle {
        account_id: bundle.account_id,
        enc_key_b64: bundle.enc_key_b64,
        mac_key_b64: bundle.mac_key_b64,
        refresh_token: bundle.refresh_token,
    }
}
