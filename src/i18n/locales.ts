export const APP_LOCALES = ["zh", "en"] as const;

export type AppLocale = (typeof APP_LOCALES)[number];

export type AppLocaleMetadata = {
  code: AppLocale;
  label: string;
  englishLabel: string;
  languageTag: string;
  collatorLocale: string;
  dateTimeLocale: string;
};

export const DEFAULT_APP_LOCALE: AppLocale = "zh";
export const FALLBACK_APP_LOCALE: AppLocale = DEFAULT_APP_LOCALE;

export const APP_LOCALE_METADATA: Record<AppLocale, AppLocaleMetadata> = {
  zh: {
    code: "zh",
    label: "中文",
    englishLabel: "Simplified Chinese",
    languageTag: "zh-Hans-CN",
    collatorLocale: "zh-Hans-CN",
    dateTimeLocale: "zh-Hans-CN",
  },
  en: {
    code: "en",
    label: "English",
    englishLabel: "English",
    languageTag: "en-US",
    collatorLocale: "en-US",
    dateTimeLocale: "en-US",
  },
};

export const APP_LOCALE_OPTIONS = APP_LOCALES.map((locale) => ({
  value: locale,
  label: APP_LOCALE_METADATA[locale].label,
}));

export function isAppLocale(value: unknown): value is AppLocale {
  return typeof value === "string" && APP_LOCALES.includes(value as AppLocale);
}

export function resolveAppLocale(
  value: unknown,
  fallback: AppLocale = FALLBACK_APP_LOCALE,
): AppLocale {
  if (typeof value !== "string") {
    return fallback;
  }

  const normalized = value.trim().toLowerCase();
  if (isAppLocale(normalized)) {
    return normalized;
  }

  if (normalized.startsWith("zh")) {
    return "zh";
  }

  if (normalized.startsWith("en")) {
    return "en";
  }

  return fallback;
}

export function getAppLocaleMetadata(
  value: unknown,
  fallback: AppLocale = FALLBACK_APP_LOCALE,
): AppLocaleMetadata {
  return APP_LOCALE_METADATA[resolveAppLocale(value, fallback)];
}
