import {
  ICON_CACHE_CLEANUP_INTERVAL_MS,
  ICON_CACHE_TTL_MS,
} from "@/features/vault/constants";

type IconCacheStatus = "loaded" | "failed";

type IconCacheEntry = {
  url: string;
  status: IconCacheStatus;
  timestamp: number;
};

/**
 * Icon cache implementation with automatic cleanup
 * Caches icon load status (loaded/failed) to prevent redundant requests
 */
class IconCache {
  private cache = new Map<string, IconCacheEntry>();
  private readonly maxAge = ICON_CACHE_TTL_MS;

  /**
   * Get cached status for an icon URL
   * Returns null if not cached or expired
   */
  get(url: string): IconCacheStatus | null {
    const entry = this.cache.get(url);
    if (!entry) {
      return null;
    }

    // Check if entry is expired
    if (Date.now() - entry.timestamp > this.maxAge) {
      this.cache.delete(url);
      return null;
    }

    return entry.status;
  }

  /**
   * Cache icon load status for a URL
   */
  set(url: string, status: IconCacheStatus): void {
    this.cache.set(url, {
      url,
      status,
      timestamp: Date.now(),
    });
  }

  /**
   * Check if URL has a valid cached entry
   */
  has(url: string): boolean {
    return this.get(url) !== null;
  }

  /**
   * Clear all cached entries
   */
  clear(): void {
    this.cache.clear();
  }

  /**
   * Remove expired entries from cache
   */
  cleanup(): void {
    const now = Date.now();
    for (const [url, entry] of this.cache.entries()) {
      if (now - entry.timestamp > this.maxAge) {
        this.cache.delete(url);
      }
    }
  }
}

// Global icon cache instance
const iconCache = new IconCache();

// Cleanup expired entries periodically
if (typeof window !== "undefined") {
  setInterval(() => {
    iconCache.cleanup();
  }, ICON_CACHE_CLEANUP_INTERVAL_MS);
}

/**
 * Hook for accessing the global icon cache
 * Provides methods to get/set cached icon load status
 */
export function useIconCache() {
  const getCachedStatus = (url: string): IconCacheStatus | null => {
    return iconCache.get(url);
  };

  const setCachedStatus = (url: string, status: IconCacheStatus): void => {
    iconCache.set(url, status);
  };

  const hasCached = (url: string): boolean => {
    return iconCache.has(url);
  };

  const clearCache = (): void => {
    iconCache.clear();
  };

  return {
    getCachedStatus,
    setCachedStatus,
    hasCached,
    clearCache,
  };
}
