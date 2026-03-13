import { createInstance } from "i18next";
import { initReactI18next } from "react-i18next";
import {
  DEFAULT_APP_LOCALE,
  FALLBACK_APP_LOCALE,
  resolveAppLocale,
} from "./locales";
import { translationResources } from "./resources";
import { loadSavedLocale, saveLocale } from "./storage";

export const appI18n = createInstance();

export async function initializeAppI18n(initialLocale?: string): Promise<void> {
  const savedLocale = await loadSavedLocale();
  const locale = resolveAppLocale(
    initialLocale ?? savedLocale,
    DEFAULT_APP_LOCALE,
  );

  await appI18n.use(initReactI18next).init({
    lng: locale,
    fallbackLng: FALLBACK_APP_LOCALE,
    defaultNS: "translation",
    resources: translationResources,
    interpolation: {
      escapeValue: false,
    },
    react: {
      useSuspense: false,
    },
  });
}

export async function changeAppLocale(locale: string): Promise<void> {
  const resolved = resolveAppLocale(locale, DEFAULT_APP_LOCALE);
  await appI18n.changeLanguage(resolved);
  await saveLocale(resolved);
}
