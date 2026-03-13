import { commands } from "@/bindings";
import {
  CUSTOM_SERVER_URL_OPTION,
  SERVER_URL_OPTIONS,
} from "@/features/auth/login/constants";
import { toErrorText } from "@/features/auth/shared/utils";
import { appI18n } from "@/i18n";

export function normalizeBaseUrl(value: string): string {
  return value.trim().replace(/\/+$/, "");
}

export function toServerUrlOption(value: string): string {
  const normalized = normalizeBaseUrl(value);
  const matched = SERVER_URL_OPTIONS.find(
    (option) => option.value === normalized,
  );
  return matched ? matched.value : CUSTOM_SERVER_URL_OPTION;
}

export function isValidServerUrl(value: string): boolean {
  try {
    const parsed = new URL(value);
    return parsed.protocol === "http:" || parsed.protocol === "https:";
  } catch {
    return false;
  }
}

export function toProviderLabel(provider: string): string {
  return appI18n.t(`auth.login.form.twoFactor.provider.providers.${provider}`, {
    defaultValue: appI18n.t(
      "auth.login.form.twoFactor.provider.providers.unknown",
      {
        provider,
      },
    ),
  });
}

export function formatTwoFactorProviders(providers: string[]): string {
  if (providers.length === 0) {
    return appI18n.t("auth.login.messages.unknownTwoFactorProvider");
  }

  const labels = providers.map(toProviderLabel);
  const separator = (appI18n.resolvedLanguage || "").startsWith("zh")
    ? "、"
    : ", ";
  return labels.join(separator);
}

export function toProviderId(provider: string): number | null {
  const parsed = Number.parseInt(provider, 10);
  if (Number.isNaN(parsed)) {
    return null;
  }
  return parsed;
}

export function toLoginErrorText(error: unknown): string {
  return toErrorText(error, appI18n.t("auth.login.messages.loginFailed"));
}

export async function restoreLoginHints() {
  return commands.authRestoreState({});
}

export async function loginWithPassword(input: {
  baseUrl: string;
  email: string;
  masterPassword: string;
  twoFactorProvider: number | null;
  twoFactorToken: string | null;
}) {
  return commands.authLoginWithPassword({
    ...input,
    twoFactorRemember: false,
    authrequest: null,
  });
}

export async function sendEmailLoginCode(input: {
  baseUrl: string;
  email: string;
  masterPassword: string;
}) {
  return commands.authSendEmailLogin({
    ...input,
    authRequestId: null,
    authRequestAccessCode: null,
  });
}

export async function canVaultUnlockAfterLogin() {
  return commands.vaultCanUnlock();
}

export async function unlockVaultAfterLogin(masterPassword: string) {
  return commands.vaultUnlock({
    method: {
      type: "masterPassword",
      password: masterPassword,
    },
  });
}

export async function syncVaultAfterLogin() {
  return commands.vaultSyncNow({
    excludeDomains: false,
  });
}
