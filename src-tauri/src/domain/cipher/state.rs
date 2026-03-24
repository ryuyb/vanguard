/// Sealed trait module to prevent external implementations
mod sealed {
    pub trait Sealed {}
}

/// Trait representing the state of a cipher (encrypted or decrypted)
///
/// This trait uses the sealed trait pattern to prevent external implementations,
/// ensuring type safety at compile time.
pub trait CipherState: sealed::Sealed {
    /// The field renaming policy for serialization
    /// - "snake_case" for encrypted ciphers (from API)
    /// - "camelCase" for decrypted ciphers (to frontend)
    const RENAME_POLICY: &'static str;

    /// Whether this state represents decrypted data
    const IS_DECRYPTED: bool;
}

/// Marker type for encrypted cipher state
#[derive(Debug, Clone, Copy, PartialEq, Eq, specta::Type)]
pub struct Encrypted;

impl sealed::Sealed for Encrypted {}

impl CipherState for Encrypted {
    const RENAME_POLICY: &'static str = "snake_case";
    const IS_DECRYPTED: bool = false;
}

/// Marker type for decrypted cipher state
#[derive(Debug, Clone, Copy, PartialEq, Eq, specta::Type)]
pub struct Decrypted;

impl sealed::Sealed for Decrypted {}

impl CipherState for Decrypted {
    const RENAME_POLICY: &'static str = "camelCase";
    const IS_DECRYPTED: bool = true;
}
