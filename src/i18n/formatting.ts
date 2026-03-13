/**
 * Locale-aware formatting helpers
 *
 * Provides shared utilities for sorting and date/time rendering
 * based on the active locale.
 */

import { appI18n } from "./app-i18n";
import { getAppLocaleMetadata } from "./locales";

/**
 * Get the current locale tag for Intl APIs
 */
function getCurrentLocaleTag(): string {
  const currentLocale = appI18n.language;
  const metadata = getAppLocaleMetadata(currentLocale);
  return metadata.languageTag;
}

/**
 * Get a locale-aware collator for string sorting
 */
export function getLocaleCollator(
  options?: Intl.CollatorOptions,
): Intl.Collator {
  const currentLocale = appI18n.language;
  const metadata = getAppLocaleMetadata(currentLocale);
  return new Intl.Collator(metadata.collatorLocale, options);
}

/**
 * Sort strings using the active locale's collation rules
 */
export function sortByLocale<T>(
  items: T[],
  selector: (item: T) => string,
): T[] {
  const collator = getLocaleCollator();
  return [...items].sort((a, b) => collator.compare(selector(a), selector(b)));
}

/**
 * Format a date using the active locale
 */
export function formatDate(
  date: Date | number | string,
  options?: Intl.DateTimeFormatOptions,
): string {
  const dateObj =
    typeof date === "string" || typeof date === "number"
      ? new Date(date)
      : date;

  const currentLocale = appI18n.language;
  const metadata = getAppLocaleMetadata(currentLocale);

  return new Intl.DateTimeFormat(metadata.dateTimeLocale, options).format(
    dateObj,
  );
}

/**
 * Format a date and time using the active locale
 */
export function formatDateTime(
  date: Date | number | string,
  options?: Intl.DateTimeFormatOptions,
): string {
  return formatDate(date, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    ...options,
  });
}

/**
 * Format a time using the active locale
 */
export function formatTime(
  date: Date | number | string,
  options?: Intl.DateTimeFormatOptions,
): string {
  return formatDate(date, {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    ...options,
  });
}

/**
 * Format a relative time (e.g., "2 hours ago")
 * Note: This is a simple implementation. For production use,
 * consider using a library like date-fns or dayjs with locale support.
 */
export function formatRelativeTime(date: Date | number | string): string {
  const dateObj =
    typeof date === "string" || typeof date === "number"
      ? new Date(date)
      : date;

  const now = new Date();
  const diffMs = now.getTime() - dateObj.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  const localeTag = getCurrentLocaleTag();

  try {
    const rtf = new Intl.RelativeTimeFormat(localeTag, { numeric: "auto" });

    if (diffDay > 0) {
      return rtf.format(-diffDay, "day");
    }
    if (diffHour > 0) {
      return rtf.format(-diffHour, "hour");
    }
    if (diffMin > 0) {
      return rtf.format(-diffMin, "minute");
    }
    return rtf.format(-diffSec, "second");
  } catch {
    // Fallback if RelativeTimeFormat is not supported
    return formatDateTime(dateObj);
  }
}
