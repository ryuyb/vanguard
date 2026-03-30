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
      yes: "Yes",
      no: "No",
    },
  },
  auth: {
    login: {
      title: "Sign in to Vanguard",
      subtitle: "Enter your credentials to access your vault",
      hero: {
        badge: "Vanguard Vault",
        title: "Welcome back, continue managing your vault",
        description:
          "Enter your server address, email, and master password to sign in and automatically prepare your vault.",
        illustrationAlt: "Vault sign-in illustration",
      },
      form: {
        serverUrl: {
          label: "Server Address",
          placeholder: "Select server address",
          customOption: "Custom Address",
          customPlaceholder: "https://vault.example.com",
        },
        email: {
          label: "Email Address",
          placeholder: "you@example.com",
        },
        masterPassword: {
          label: "Master Password",
          placeholder: "Enter master password",
          showPassword: "Show password",
          hidePassword: "Hide password",
        },
        twoFactor: {
          title: "Two-Factor Authentication",
          provider: {
            label: "Authentication Method",
            placeholder: "Select authentication method",
            providers: {
              "0": "Authenticator",
              "1": "Email",
              "2": "Duo",
              "3": "YubiKey",
              "5": "Remember Me",
              "7": "WebAuthn",
              "8": "Recovery Code",
              unknown: "Provider {{provider}}",
            },
          },
          token: {
            label: "Two-Factor Code",
            placeholder: "Enter verification code",
          },
          sendEmail: "Send Email Code",
          sendingEmail: "Sending email code...",
        },
      },
      actions: {
        submit: "Sign In",
        submitting: "Signing in...",
        verifyAndContinue: "Verify and Continue",
      },
      states: {
        checkingSession: "Checking session...",
      },
      validation: {
        missingCredentials:
          "Enter the server address, email, and master password first.",
        missingServerUrl: "Please enter server address.",
        missingEmail: "Please enter email address.",
        missingPassword: "Please enter master password.",
        invalidServerUrl:
          "The server address is invalid. It must start with http:// or https://.",
        invalidEmail:
          "The email address looks invalid. Please check it and try again.",
        incompleteTwoFactor:
          "Enter a complete two-factor verification code before continuing.",
        nonEmailProvider:
          "The selected method is not email verification, so an email code cannot be sent.",
        missingEmailCodeRequirements:
          "Enter the server address, email, and master password before sending an email code.",
      },
      progress: {
        verifyingAccount: "Verifying account...",
        preparingVault: "Preparing your vault...",
        unlockingLocalVault: "Unlocking local vault...",
        syncingLatestData: "Syncing latest data...",
        firstSync: "Performing initial vault sync...",
        finishingUnlock: "Finishing unlock...",
      },
      messages: {
        loginFailed: "Sign in failed. Please try again later.",
        twoFactorPrompt:
          "Two-factor authentication is required. Enter the code to continue (available methods: {{providers}}).",
        unknownTwoFactorProvider: "unknown method",
        emailCodeSent:
          "An email verification code was sent. Check your inbox, then enter it to continue signing in.",
      },
    },
    unlock: {
      title: "Unlock Vanguard",
      subtitle: "Enter your credentials to access your vault",
      hero: {
        badge: "Vault Unlock",
        title: "Session locked, please enter master password to unlock",
        description:
          "Enter your master password to unlock and continue securely accessing your vault.",
        illustrationAlt: "Vault locked illustration",
      },
      form: {
        account: {
          label: "Account",
          unknown: "Unknown",
        },
        server: {
          label: "Server",
          unknown: "Unknown",
        },
        pin: {
          label: "PIN Code",
          placeholder: "Enter PIN to unlock",
        },
        masterPassword: {
          label: "Master Password",
          placeholder: "Enter master password to unlock",
          showPassword: "Show password",
          hidePassword: "Hide password",
        },
      },
      actions: {
        unlock: "Unlock",
        unlocking: "Unlocking...",
        unlockWithPin: "Unlock with PIN",
        unlockingWithPin: "Unlocking...",
        switchToMasterPassword: "Use Master Password",
        switchToPin: "Use PIN",
        biometric: "Use Biometric",
        biometricVerifying: "Verifying...",
        logout: "Sign Out",
        loggingOut: "Signing out...",
        goToLogin: "Go to Sign In",
      },
      states: {
        checkingSession: "Checking session status...",
        unlocked: "Vault unlocked, redirecting...",
        needsLogin: {
          title: "Sign In Required",
          description: "No session available to unlock, please sign in first.",
        },
        biometricUnavailable: {
          title: "Biometric Unavailable",
          description:
            "No local data available for unlocking on this device. Please unlock with password and complete sync first.",
        },
      },
      validation: {
        sessionNotLocked:
          "The current session is not locked, so it cannot be unlocked.",
        missingMasterPassword: "Enter your master password.",
        pinNotEnabled:
          "PIN unlock is not enabled for this account. Use your master password instead.",
        missingPin: "Enter your PIN.",
        sessionNotLockedBiometric:
          "The current session is not locked, so biometric unlock is unavailable.",
      },
      messages: {
        unlockFailed: "Unlock failed. Please try again later.",
      },
    },
    register: {
      title: "Create Vanguard Account",
      subtitle: "Register a new account to start using your vault",
      hero: {
        badge: "Account Registration",
        title: "Create a new account and start managing your passwords",
        description:
          "Enter your server address and email to create a new vault account.",
        illustrationAlt: "Account registration illustration",
      },
      form: {
        serverUrl: {
          label: "Server Address",
          placeholder: "Select server address",
          customOption: "Custom Address",
          customPlaceholder: "https://vault.example.com",
        },
        email: {
          label: "Email Address",
          placeholder: "you@example.com",
        },
        name: {
          label: "Name",
          placeholder: "Enter your name",
        },
        masterPassword: {
          label: "Master Password",
          placeholder: "Create master password",
          showPassword: "Show password",
          hidePassword: "Hide password",
        },
        confirmPassword: {
          label: "Confirm Password",
          placeholder: "Re-enter master password",
        },
        passwordHint: {
          label: "Password Hint (Optional)",
          placeholder: "A hint to help you remember your password",
        },
      },
      actions: {
        submit: "Create Account",
        submitting: "Creating account...",
        backToLogin: "Back to Sign In",
        finishRegistration: "Finish Registration",
        finishing: "Finishing registration...",
      },
      passwordSetup: {
        title: "Set Master Password",
        subtitle: "Create master password for {{email}}",
      },
      emailVerification: {
        title: "Check Your Email",
        description:
          "Click the link in the email sent to {{email}} to continue creating your account.",
        notReceived: "Didn't receive the email?",
        editEmail: "Go back to edit your email address.",
        backToLogin: "Back to Sign In",
      },
      validation: {
        missingServerUrl: "Please enter server address.",
        invalidServerUrl:
          "The server address is invalid. It must start with http:// or https://.",
        missingEmail: "Please enter email address.",
        invalidEmail:
          "The email address looks invalid. Please check it and try again.",
        missingName: "Please enter your name.",
        missingPassword: "Please enter master password.",
        passwordTooShort: "Master password must be at least 8 characters.",
        passwordMismatch: "Passwords do not match.",
      },
      progress: {
        creatingAccount: "Creating account...",
        sendingVerification: "Sending verification email...",
        generatingKeys: "Generating encryption keys...",
        finishingRegistration: "Finishing registration...",
        loggingIn: "Signing in...",
        loginSuccess: "Sign in successful",
        syncingVaultData: "Syncing vault data...",
        unlockingVault: "Unlocking vault...",
      },
      messages: {
        registrationDisabled: {
          title: "Registration Disabled",
          description:
            "This server does not allow new user registration. Please contact the administrator.",
        },
        emailVerificationRequired: {
          title: "Email Verification Required",
          description:
            "A verification email has been sent to your inbox. Please check your email and click the link to complete registration. After that, return to the sign-in page to log in.",
        },
        registrationFailed: "Registration failed. Please try again later.",
        passwordStrength: {
          weak: "Weak",
          fair: "Fair",
          good: "Good",
          strong: "Strong",
        },
        passwordCompromised:
          "This password has appeared in a data breach. Consider using a different password.",
        checkingPassword: "Checking password security...",
      },
    },
    feedback: {
      login: {
        error: "Sign In Failed",
        success: "Sign In Successful",
        twoFactorRequired: "Two-Factor Authentication Required",
      },
      unlock: {
        error: "Verification Failed",
      },
      register: {
        error: "Registration Failed",
        success: "Registration Successful",
      },
    },
  },
  vault: {
    settings: {},
    page: {
      user: {
        notSignedIn: "Not signed in",
        unknownService: "Unknown service",
      },
      actions: {
        create: "Create",
        rename: "Rename",
        delete: "Delete",
        edit: "Edit",
        settings: "Settings",
        lock: "Lock",
        locking: "Locking...",
        logout: "Log Out",
        loggingOut: "Logging out...",
        restore: "Restore",
        restoring: "Restoring...",
        permanentDelete: "Delete permanently",
      },
      cipher: {
        create: "Create item",
        untitled: "Untitled item",
        contextMenu: {
          view: "View",
          edit: "Edit",
          clone: "Clone",
        },
        cloneSuffix: "Copy",
      },
      filters: {
        ariaLabel: "Filter items",
        types: {
          all: "All types",
          login: "Login",
          card: "Card",
          identity: "Identity",
          note: "Note",
          sshKey: "SSH key",
        },
      },
      folders: {
        title: "Folders",
        untitledFolder: "Untitled folder",
        expandFolder: "Expand folder",
        collapseFolder: "Collapse folder",
      },
      menus: {
        allItems: "All Items",
        favorites: "Favorites",
        trash: "Trash",
        send: "Send",
        noFolder: "No Folder",
        unknownFolder: "Unknown folder",
      },
      search: {
        placeholder: "Search vault",
        inlinePlaceholder: "Search in {{menu}}",
        close: "Close search",
        searching: "Searching...",
        resultCount: "{{count}} result(s)",
      },
      sort: {
        ariaLabel: "Sort items",
        byLabel: "Sort by",
        by: {
          title: "Title",
          created: "Created date",
          modified: "Modified date",
        },
        directionLabel: "Order",
        direction: {
          alphaAsc: "A to Z",
          alphaDesc: "Z to A",
          newestFirst: "Newest first",
          oldestFirst: "Oldest first",
        },
      },
      states: {
        loading: "Loading vault...",
        loadError: "Failed to load vault data.",
        emptyFiltered: "No items match the current filters.",
        loadingCipherDetail: "Loading item details...",
        sessionExpired: {
          title: "Session Expired",
          description:
            "API session has expired, local data is still accessible. Please unlock with master password to restore sync functionality.",
        },
      },
    },
    dialogs: {
      folder: {
        createTitle: "Create folder",
        createDescription: "Create a new folder to organize your items.",
        createSubFolderTitle: "Create subfolder",
        createSubFolderDescription:
          "Create a subfolder under {{parentFolderName}}.",
        renameTitle: "Rename folder",
        renameDescription: "Update the folder name.",
        fullPathLabel: "Full path:",
        folderNameLabel: "Folder name",
        subFolderNameLabel: "Subfolder name",
        folderNamePlaceholder: "Enter folder name",
        subFolderNamePlaceholder: "Enter subfolder name",
        processing: "Processing...",
      },
      deleteFolder: {
        title: "Delete folder",
        descriptionPrefix: "Are you sure you want to delete",
        descriptionSuffix: "?",
        descriptionHint:
          "Items in this folder will not be deleted, but they will no longer belong to this folder.",
        deleting: "Deleting...",
      },
      deleteCipher: {
        title: "Delete item",
        descriptionPrefix: "Are you sure you want to delete",
        descriptionSuffix: "?",
        deleting: "Deleting...",
      },
      restoreCipher: {
        title: "Restore item",
        descriptionPrefix: "Are you sure you want to restore",
        descriptionSuffix: "?",
        confirming: "Restoring...",
      },
      permanentDeleteCipher: {
        title: "Permanently delete item",
        descriptionPrefix: "Are you sure you want to permanently delete",
        descriptionSuffix: "? This action cannot be undone.",
        confirming: "Deleting...",
      },
      cipherForm: {
        createTitle: "Create item",
        createDescription: "Add a new item to your vault.",
        editTitle: "Edit item",
        editDescription: "Update item details.",
        fields: {
          type: "Type",
          name: "Name",
          folder: "Folder",
          username: "Username",
          password: "Password",
          totp: "TOTP",
          uris: "Website URLs",
          cardholderName: "Cardholder name",
          cardNumber: "Card number",
          cardBrand: "Card brand",
          expMonth: "Expiration month",
          expYear: "Expiration year",
          securityCode: "Security code",
          sshPrivateKey: "SSH private key",
          sshPublicKey: "SSH public key",
          sshFingerprint: "SSH fingerprint",
          notes: "Notes",
          customFields: "Custom fields",
        },
        placeholders: {
          name: "Enter item name",
          username: "Enter username",
          password: "Enter password",
          totp: "Enter TOTP secret",
          uri: "https://example.com",
          notes: "Add notes",
          cardholderName: "Name on card",
          cardBrand: "Select card brand",
          cardNumber: "1234 5678 9012 3456",
          month: "Month",
          year: "Year",
          securityCode: "CVV/CVC",
          sshPrivateKey: "-----BEGIN OPENSSH PRIVATE KEY-----",
          sshPublicKey: "ssh-rsa AAAA...",
          sshFingerprint: "SHA256:...",
          customFieldName: "Field name",
          customFieldValue: "Field value",
          hiddenValue: "Hidden value",
          linkValue: "Linked value",
          booleanValue: "Select true or false",
        },
        types: {
          login: "Login",
          note: "Secure note",
          card: "Card",
          sshKey: "SSH key",
        },
        brands: {
          visa: "Visa",
          mastercard: "Mastercard",
          americanExpress: "American Express",
          discover: "Discover",
          jcb: "JCB",
          unionPay: "UnionPay",
          other: "Other",
        },
        customFieldTypes: {
          text: "Text",
          hidden: "Hidden",
          boolean: "Boolean",
          linked: "Linked",
        },
        actions: {
          addUri: "Add URL",
          addCustomField: "Add custom field",
          creating: "Creating...",
          saving: "Saving...",
          pasteQr: "Paste QR from clipboard",
        },
        validation: {
          nameRequired: "Name is required.",
        },
        noFolder: "No folder",
      },
      settings: {
        title: "Vault settings",
        description:
          "Manage general preferences and security behavior for your vault.",
        sections: {
          general: "General",
          security: "Security",
        },
        general: {
          title: "General",
          description: "Set appearance and basic behavior preferences.",
          launchOnLogin: "Launch app on login",
          showWebsiteIcon: "Show website icon",
          spotlightAutofill: "Spotlight autofill",
          spotlightAutofillDescription:
            "Automatically fill copied credentials into the previous input field",
          shortcuts: {
            title: "Keyboard shortcuts",
            quickAccess: "Quick access shortcut",
            lock: "Lock vault shortcut",
            unset: "Not set",
            inputHint: "Press keys to record shortcut",
            clear: "Clear",
            clearQuickAccess: "Clear quick access shortcut",
            clearLock: "Clear lock shortcut",
            keys: {
              space: "Space",
              esc: "Esc",
              up: "Up",
              down: "Down",
              left: "Left",
              right: "Right",
            },
          },
        },
        security: {
          unlock: {
            title: "Unlock",
            description: "Configure unlock methods.",
          },
          biometric: {
            label: "Biometric unlock",
            enabledHint: "Use device biometric authentication to unlock.",
            checkingHint: "Checking biometric availability...",
          },
          pin: {
            label: "PIN unlock",
            enabledHint: "Use a PIN to unlock quickly.",
            unsupportedHint: "PIN unlock is not available on this device.",
          },
          requireMasterPassword: "Require master password after",
          autoLock: {
            title: "Auto-lock",
            description: "Automatically lock your vault after inactivity.",
          },
          lockOnSleep: "Lock when device sleeps",
          idleLockDelay: "Lock after idle time",
          clipboard: {
            title: "Clipboard",
            description: "Control how copied secrets are cleared.",
            clearAfter: "Clear clipboard after",
          },
        },
        placeholders: {
          language: "Select language",
          requireMasterPassword: "Select when to require master password",
          autoLockIdle: "Select idle lock delay",
          clipboardClear: "Select clipboard clear delay",
        },
        options: {
          requireMasterPassword: {
            "1d": "1 day",
            "7d": "7 days",
            "14d": "14 days",
            "30d": "30 days",
            never: "Never",
          },
          autoLockIdle: {
            "1m": "1 minute",
            "2m": "2 minutes",
            "5m": "5 minutes",
            "10m": "10 minutes",
            "15m": "15 minutes",
            "30m": "30 minutes",
            "1h": "1 hour",
            "4h": "4 hours",
            "8h": "8 hours",
            never: "Never",
          },
          clipboardClear: {
            "10s": "10 seconds",
            "20s": "20 seconds",
            "30s": "30 seconds",
            "1m": "1 minute",
            "2m": "2 minutes",
            "5m": "5 minutes",
            never: "Never",
          },
        },
        pinDialog: {
          title: "Enable PIN unlock",
          description:
            "Set a PIN for quick unlock. Keep it memorable and secure.",
          pinPlaceholder: "Enter PIN",
          enabling: "Enabling...",
        },
        errors: {
          loadBiometricStatus: "Failed to load biometric status.",
          loadPinStatus: "Failed to load PIN status.",
          loadSecuritySettings: "Failed to load security settings.",
          enableBiometric: "Failed to enable biometric unlock.",
          disableBiometric: "Failed to disable biometric unlock.",
          enablePin: "Failed to enable PIN unlock.",
          disablePin: "Failed to disable PIN unlock.",
          pinRequired: "PIN is required.",
          saveFailed: "Failed to save settings.",
          spotlightAutofillPermission: "Accessibility permission required",
          spotlightAutofillPermissionDescription:
            "Please grant Vanguard accessibility permission in System Settings > Privacy & Security > Accessibility to use the autofill feature.",
        },
      },
    },
    detail: {
      unknown: "Unknown",
      fields: {
        username: "Username",
        password: "Password",
        oneTimePassword: "One-time password",
        passkey: "Passkey",
        organization: "Organization",
        uris: "URLs",
        notes: "Notes",
        customFields: "Custom fields",
      },
      actions: {
        showPassword: "Show password",
        hidePassword: "Hide password",
        showFieldValue: "Show field value",
        hideFieldValue: "Hide field value",
      },
      boolean: {
        true: "True",
        false: "False",
      },
      customFields: {
        unnamedField: "Unnamed field",
        emptyValue: "(empty)",
      },
      passkey: {
        createdAt: "Created on {{date}}",
      },
      timeline: {
        lastEdited: "Last edited",
        created: "Created",
        passwordUpdated: "Password updated",
        passkeyCreated: "Passkey created",
        archived: "Archived",
        deleted: "Deleted",
        lastEditedWithValue: "Last edited: {{date}}",
        empty: "No timeline data available.",
      },
      totp: {
        countdownAria: "TOTP countdown",
        countdownTitle: "Time remaining for current TOTP code",
      },
    },
    feedback: {
      loadError: "Failed to load vault data. Please try again.",
      copiedToClipboard: "Copied to clipboard",
      qrParsed: "TOTP secret extracted from QR code",
      qrNoImage: "No image found in clipboard",
      qrDecodeFailed: "Could not decode QR code from clipboard image",
      qrNoTotp: "QR code does not contain a TOTP secret",
      iconAlt: "{{name}} icon",
      iconAltFallback: "Vault item icon",
      folder: {
        createSuccess: {
          title: "Folder created",
          description: '"{{name}}" has been created.',
        },
        createError: {
          title: "Failed to create folder",
          description: "Unable to create folder. Please try again.",
        },
        renameSuccess: {
          title: "Folder renamed",
          description: 'Folder renamed to "{{name}}".',
        },
        renameError: {
          title: "Failed to rename folder",
          description: "Unable to rename folder. Please try again.",
        },
        deleteSuccess: {
          title: "Folder deleted",
          description: '"{{name}}" has been deleted.',
        },
        deleteError: {
          title: "Failed to delete folder",
          description: "Unable to delete folder. Please try again.",
        },
      },
      cipher: {
        createSuccess: {
          title: "Item created",
          description: '"{{name}}" has been created.',
        },
        createError: {
          title: "Failed to create item",
          description: "Unable to create item. Please try again.",
        },
        saveSuccess: {
          title: "Item saved",
          description: '"{{name}}" has been updated.',
        },
        saveError: {
          title: "Failed to save item",
          description: "Unable to save item. Please try again.",
        },
        deleteSuccess: {
          title: "Item deleted",
          description: '"{{name}}" has been deleted.',
        },
        deleteError: {
          title: "Failed to delete item",
          description: "Unable to delete item. Please try again.",
        },
        restoreSuccess: {
          title: "Item restored",
          description: '"{{name}}" has been restored.',
        },
        restoreError: {
          title: "Failed to restore item",
          description: "Unable to restore item. Please try again.",
        },
        permanentDeleteSuccess: {
          title: "Item permanently deleted",
          description: '"{{name}}" has been permanently deleted.',
        },
        permanentDeleteError: {
          title: "Failed to permanently delete item",
          description: "Unable to permanently delete item. Please try again.",
        },
      },
    },
  },
  spotlight: {
    search: {
      ariaLabel: "Search",
      placeholder: "Search vault...",
      detailRegionLabel: "Cipher detail",
      states: {
        noResults: {
          title: "No matching items",
          description: "Try another keyword.",
        },
      },
    },
    hints: {
      copyUsername: "Copy Username",
      copyPassword: "Copy Password",
      moreActions: "More Actions",
      backToResults: "Back to Results",
      select: "Select",
      openShortcut: "Open quick access",
      close: "Close",
    },
    actions: {
      copyUsername: "Copy Username",
      copyPassword: "Copy Password",
      copyTotp: "Copy TOTP",
    },
    items: {
      untitledCipher: "Untitled Cipher",
      defaultSubtitle: "Vault item",
    },
    states: {
      unknownError: "Unknown error",
    },
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
  send: {
    types: {
      all: "All types",
      text: "Text",
      file: "File",
    },
    list: {
      create: "Create Send",
      untitled: "Untitled send",
      disabled: "Disabled",
      expired: "Expired",
      expires: "Expires",
      views: "views",
      noExpiration: "No expiration",
      expiresOn: "Expires {{date}}",
      viewCount: "{{count}}/{{max}} views",
      empty: {
        title: "No sends yet",
        description: "Create a send to share text or files securely",
        action: "Create Send",
      },
    },
    form: {
      createTitle: "Create Send",
      editTitle: "Edit Send",
      createDescription: "Create a new send to share securely",
      editDescription: "Edit send details",
      type: "Type",
      name: "Name",
      textContent: "Text Content",
      hideText: "Hide text by default",
      file: "File",
      chooseFile: "Choose file...",
      notes: "Notes",
      advanced: "Advanced Options",
      password: "Password",
      removePassword: "Remove",
      maxAccessCount: "Max access count",
      expirationDate: "Expiration date",
      deletionDate: "Deletion date",
      hideEmail: "Hide my email",
      disable: "Disable this send",
      submit: {
        create: "Create Send",
        save: "Save",
      },
    },
    feedback: {
      createSuccess: "Send created",
      createError: "Failed to create send",
      saveSuccess: "Send saved",
      saveError: "Failed to save send",
      deleteSuccess: "Send deleted",
      deleteError: "Failed to delete send",
      removePasswordSuccess: "Password removed",
    },
    dialogs: {
      delete: {
        title: "Delete Send",
        descriptionPrefix: "Are you sure you want to delete",
        descriptionSuffix: "? This action cannot be undone.",
        deleting: "Deleting...",
      },
      removePassword: {
        title: "Remove Password",
        description:
          "Are you sure you want to remove the password from this Send?",
        confirm: "Remove",
      },
    },
    contextMenu: {
      edit: "Edit",
      copyLink: "Copy link",
      delete: "Delete",
    },
    detail: {
      selectPrompt: "Select a send to view details",
      textContent: "Text Content",
      fileInfo: "File",
      notes: "Notes",
      sendLink: "Send Link",
      details: "Details",
      password: "Password",
      passwordProtected: "Protected",
      passwordNone: "None",
      maxViews: "Max views",
      currentViews: "Current views",
      hideEmail: "Hide email",
      disabled: "Disabled",
      yes: "Yes",
      no: "No",
      dates: "Dates",
      expiration: "Expiration",
      deletion: "Deletion",
      lastUpdated: "Last updated",
      noExpiration: "No expiration",
      linkCopied: "Send link copied",
    },
  },
};
