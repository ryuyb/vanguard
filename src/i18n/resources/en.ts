import type { AppTranslationCatalog } from "./types";

export const enTranslationCatalog: AppTranslationCatalog = {
  common: {
    app: {
      name: "Vanguard",
    },
    locale: {
      label: "Language",
      options: {
        zh: "中文",
        en: "English",
      },
    },
    actions: {
      cancel: "Cancel",
      confirm: "Confirm",
      close: "Close",
      save: "Save",
    },
    states: {
      loading: "Loading...",
      unavailable: "Unavailable",
    },
  },
  auth: {
    login: {},
    unlock: {},
    feedback: {},
  },
  vault: {
    settings: {},
    page: {},
    dialogs: {},
    detail: {},
    feedback: {},
  },
  spotlight: {
    search: {},
    hints: {},
    actions: {},
  },
  errors: {
    common: {
      unknown: {
        title: "Unknown Error",
        description: "An unknown error occurred, please try again",
      },
      action: "Action",
    },
    auth: {
      invalidCredentials: {
        title: "Login Failed",
        description: "Invalid username or password, please check and retry",
      },
      tokenExpired: {
        title: "Session Expired",
        description: "Please log in again",
        action: "Log In",
      },
      tokenInvalid: {
        title: "Authentication Failed",
        description: "Invalid authentication information, please log in again",
        action: "Log In",
      },
      permissionDenied: {
        title: "Permission Denied",
        description: "You do not have permission to perform this action",
      },
      accountLocked: {
        title: "Account Locked",
        description:
          "Your account has been locked, please contact administrator",
      },
      twoFactorRequired: {
        title: "Two-Factor Authentication Required",
        description: "Please enter your two-factor authentication code",
      },
      invalidPin: {
        title: "Invalid PIN",
        description: "The PIN is incorrect, please try again",
      },
    },
    vault: {
      cipherNotFound: {
        title: "Item Not Found",
        description: "The item was not found, it may have been deleted",
      },
      decryptionFailed: {
        title: "Decryption Failed",
        description:
          "Unable to decrypt data, please check your master password",
      },
      syncConflict: {
        title: "Sync Conflict",
        description: "Data conflict detected, please resolve manually",
      },
      locked: {
        title: "Vault Locked",
        description: "Please unlock the vault first",
        action: "Unlock",
      },
      corrupted: {
        title: "Data Corrupted",
        description: "Vault data is corrupted, please contact support",
      },
    },
    validation: {
      fieldError: {
        title: "Invalid Input",
        description: "Please check if the input data is correct",
      },
      formatError: {
        title: "Format Error",
        description: "Data format is incorrect, please re-enter",
      },
      required: {
        title: "Required Fields Missing",
        description: "Please fill in all required fields",
      },
    },
    network: {
      connectionFailed: {
        title: "Network Connection Failed",
        description: "Unable to connect to server, please check your network",
      },
      timeout: {
        title: "Request Timeout",
        description: "Server response timeout, please try again later",
      },
      remoteError: {
        title: "Server Error",
        description: "Server returned an error, please try again later",
      },
      dnsResolutionFailed: {
        title: "Connection Failed",
        description:
          "Unable to resolve server address, please check network settings",
      },
    },
    storage: {
      databaseError: {
        title: "Data Save Failed",
        description: "Unable to save data, please try again",
      },
      fileNotFound: {
        title: "File Not Found",
        description: "The requested file does not exist",
      },
      permissionDenied: {
        title: "Permission Denied",
        description: "No permission to access this file",
      },
    },
    crypto: {
      keyDerivationFailed: {
        title: "Key Generation Failed",
        description: "Unable to generate encryption key",
      },
      encryptionFailed: {
        title: "Encryption Failed",
        description: "Data encryption failed, please try again",
      },
      decryptionFailed: {
        title: "Decryption Failed",
        description: "Unable to decrypt data, please check your password",
      },
      invalidKey: {
        title: "Invalid Key",
        description: "Encryption key is invalid, cannot continue",
      },
    },
    internal: {
      unexpected: {
        title: "Unexpected Error",
        description: "An unexpected error occurred, please contact support",
      },
      notImplemented: {
        title: "Feature Not Implemented",
        description: "This feature is not yet implemented",
      },
    },
  },
};
