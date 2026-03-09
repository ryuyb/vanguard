type IconCacheStatus = "loaded" | "failed";

type IconCacheEntry = {
  url: string;
  status: IconCacheStatus;
  timestamp: number;
};

class IconCache {
  private cache = new Map<string, IconCacheEntry>();
  private readonly maxAge = 1000 * 60 * 60; // 1 hour

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

  set(url: string, status: IconCacheStatus): void {
    this.cache.set(url, {
      url,
      status,
      timestamp: Date.now(),
    });
  }

  has(url: string): boolean {
    return this.get(url) !== null;
  }

  clear(): void {
    this.cache.clear();
  }

  // Clean up expired entries
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

// Cleanup expired entries every 5 minutes
if (typeof window !== "undefined") {
  setInterval(
    () => {
      iconCache.cleanup();
    },
    1000 * 60 * 5,
  );
}

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
