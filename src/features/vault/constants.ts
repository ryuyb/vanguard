export const ALL_ITEMS_ID = "__all_items__";
export const FAVORITES_ID = "__favorites__";
export const TRASH_ID = "__trash__";
export const NO_FOLDER_ID = "__no_folder__";

// Icon server configuration
export const DEFAULT_ICON_SERVER = "https://icons.bitwarden.net";

// Icon cache configuration
export const ICON_CACHE_TTL_MS = 1000 * 60 * 60; // 1 hour
export const ICON_CACHE_CLEANUP_INTERVAL_MS = 1000 * 60 * 5; // 5 minutes

// Icon observer configuration
export const ICON_OBSERVER_CONFIG = {
  rootMargin: "120px 0px",
  threshold: 0.01,
} as const;
