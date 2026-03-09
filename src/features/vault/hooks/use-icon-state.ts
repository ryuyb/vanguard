import { useCallback, useState } from "react";
import { useIconCache } from "@/features/vault/hooks/use-icon-cache";
import type { CipherIconLoadState } from "@/features/vault/types";

export type IconStateMap = Record<string, CipherIconLoadState>;

export function useIconState() {
  const [iconLoadStates, setIconLoadStates] = useState<IconStateMap>({});
  const [visibleCipherIds, setVisibleCipherIds] = useState<Set<string>>(
    new Set(),
  );
  const { getCachedStatus, setCachedStatus } = useIconCache();

  const updateIconLoadState = useCallback(
    (cipherId: string, state: CipherIconLoadState) => {
      setIconLoadStates((previous) => {
        if (previous[cipherId] === state) {
          return previous;
        }
        return {
          ...previous,
          [cipherId]: state,
        };
      });
    },
    [],
  );

  const setIconLoading = useCallback(
    (cipherId: string) => {
      updateIconLoadState(cipherId, "loading");
    },
    [updateIconLoadState],
  );

  const setIconLoaded = useCallback(
    (cipherId: string, iconUrl?: string) => {
      updateIconLoadState(cipherId, "loaded");
      if (iconUrl) {
        setCachedStatus(iconUrl, "loaded");
      }
    },
    [setCachedStatus, updateIconLoadState],
  );

  const setIconFallback = useCallback(
    (cipherId: string, iconUrl?: string) => {
      updateIconLoadState(cipherId, "fallback");
      if (iconUrl) {
        setCachedStatus(iconUrl, "failed");
      }
    },
    [setCachedStatus, updateIconLoadState],
  );

  const setCipherVisible = useCallback((cipherId: string, visible: boolean) => {
    setVisibleCipherIds((previous) => {
      if (visible && previous.has(cipherId)) {
        return previous;
      }
      if (!visible && !previous.has(cipherId)) {
        return previous;
      }
      const next = new Set(previous);
      if (visible) {
        next.add(cipherId);
      } else {
        next.delete(cipherId);
      }
      return next;
    });
  }, []);

  const cleanupStaleStates = useCallback((activeCipherIds: string[]) => {
    const activeCipherIdSet = new Set(activeCipherIds);

    setVisibleCipherIds((previous) => {
      const next = new Set<string>();
      for (const cipherId of previous) {
        if (activeCipherIdSet.has(cipherId)) {
          next.add(cipherId);
        }
      }
      return next.size === previous.size ? previous : next;
    });

    setIconLoadStates((previous) => {
      const next: IconStateMap = {};
      let hasChanges = false;

      for (const cipherId of activeCipherIds) {
        next[cipherId] = previous[cipherId] ?? "idle";
      }

      const previousKeys = Object.keys(previous);
      if (previousKeys.length !== activeCipherIds.length) {
        hasChanges = true;
      } else {
        for (const key of previousKeys) {
          if (!activeCipherIdSet.has(key)) {
            hasChanges = true;
            break;
          }
        }
      }

      return hasChanges ? next : previous;
    });
  }, []);

  const getIconLoadState = useCallback(
    (cipherId: string, iconUrl?: string): CipherIconLoadState => {
      // Check cache first
      if (iconUrl) {
        const cachedStatus = getCachedStatus(iconUrl);
        if (cachedStatus === "loaded") {
          return "loaded";
        }
        if (cachedStatus === "failed") {
          return "fallback";
        }
      }

      return iconLoadStates[cipherId] ?? "idle";
    },
    [getCachedStatus, iconLoadStates],
  );

  const isVisible = useCallback(
    (cipherId: string): boolean => {
      return visibleCipherIds.has(cipherId);
    },
    [visibleCipherIds],
  );

  return {
    iconLoadStates,
    visibleCipherIds,
    updateIconLoadState,
    setIconLoading,
    setIconLoaded,
    setIconFallback,
    setCipherVisible,
    cleanupStaleStates,
    getIconLoadState,
    isVisible,
    getCachedStatus,
  };
}
