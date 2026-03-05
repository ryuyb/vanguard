import {
  CUSTOM_SERVER_URL_OPTION,
  SERVER_URL_OPTIONS,
  TWO_FACTOR_PROVIDER_LABELS,
} from "@/features/auth/login/constants";

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
  return TWO_FACTOR_PROVIDER_LABELS[provider] ?? `Provider ${provider}`;
}

export function toProviderId(provider: string): number | null {
  const parsed = Number.parseInt(provider, 10);
  if (Number.isNaN(parsed)) {
    return null;
  }
  return parsed;
}
