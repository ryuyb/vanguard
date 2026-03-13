import {
  APP_LOCALE_STORAGE_KEY,
  type AppLocale,
  resolveAppLocale,
} from "./locales";

export function loadSavedLocale(): AppLocale | null {
  try {
    const saved = localStorage.getItem(APP_LOCALE_STORAGE_KEY);
    if (!saved) {
      return null;
    }
    return resolveAppLocale(saved);
  } catch {
    return null;
  }
}

export function saveLocale(locale: AppLocale): void {
  try {
    localStorage.setItem(APP_LOCALE_STORAGE_KEY, locale);
  } catch {
    // Ignore storage errors
  }
}

export function clearSavedLocale(): void {
  try {
    localStorage.removeItem(APP_LOCALE_STORAGE_KEY);
  } catch {
    // Ignore storage errors
  }
}
