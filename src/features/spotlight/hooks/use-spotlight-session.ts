import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useState } from "react";
import { commands } from "@/bindings";
import { logClientError } from "@/features/spotlight/logging";
import type { SpotlightItem } from "@/features/spotlight/types";
import { toCipherItem } from "@/features/spotlight/utils";
import { resolveSessionRoute } from "@/lib/route-session";

type UseSpotlightSessionResult = {
  hideSpotlight: () => Promise<void>;
  isLoadingVault: boolean;
  refreshSpotlightState: () => Promise<void>;
  vaultItems: SpotlightItem[];
};

export function useSpotlightSession(): UseSpotlightSessionResult {
  const [vaultItems, setVaultItems] = useState<SpotlightItem[]>([]);
  const [isLoadingVault, setIsLoadingVault] = useState(false);

  const hideSpotlight = useCallback(async () => {
    try {
      await getCurrentWindow().hide();
    } catch (error) {
      logClientError("Failed to hide spotlight", error);
    }
  }, []);

  const openMainWindow = useCallback(async () => {
    try {
      const result = await commands.desktopOpenMainWindow();
      if (result.status === "error") {
        logClientError(
          "Failed to open main window via desktop command",
          result.error,
        );
      }
    } catch (error) {
      logClientError("Failed to open main window via desktop command", error);
    } finally {
      await hideSpotlight();
    }
  }, [hideSpotlight]);

  const ensureSpotlightSession = useCallback(async () => {
    try {
      const target = await resolveSessionRoute();
      if (target === "/vault") {
        return true;
      }

      await openMainWindow();
      return false;
    } catch (error) {
      logClientError("Failed to resolve spotlight session route", error);
      await hideSpotlight();
      return false;
    }
  }, [hideSpotlight, openMainWindow]);

  const loadVaultItems = useCallback(async () => {
    setIsLoadingVault(true);

    try {
      const viewData = await commands.vaultGetViewData();
      if (viewData.status === "error") {
        logClientError("Failed to load vault data", viewData.error);
        setVaultItems([]);
        return;
      }

      const ciphers = viewData.data.ciphers.map(toCipherItem);
      setVaultItems(ciphers);
    } catch (error) {
      logClientError("Failed to load vault data", error);
      setVaultItems([]);
    } finally {
      setIsLoadingVault(false);
    }
  }, []);

  const refreshSpotlightState = useCallback(async () => {
    const canUseSpotlight = await ensureSpotlightSession();
    if (!canUseSpotlight) {
      return;
    }
    await loadVaultItems();
  }, [ensureSpotlightSession, loadVaultItems]);

  useEffect(() => {
    void refreshSpotlightState();
  }, [refreshSpotlightState]);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let disposed = false;

    void getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (!focused) {
          return;
        }
        void refreshSpotlightState();
      })
      .then((unsubscribe) => {
        if (disposed) {
          unsubscribe();
          return;
        }
        unlisten = unsubscribe;
      })
      .catch((error) => {
        logClientError("Failed to subscribe spotlight focus events", error);
      });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [refreshSpotlightState]);

  useEffect(() => {
    const handleWindowFocus = () => {
      void refreshSpotlightState();
    };
    const handleVisibilityChange = () => {
      if (document.visibilityState !== "visible") {
        return;
      }
      void refreshSpotlightState();
    };

    window.addEventListener("focus", handleWindowFocus);
    window.addEventListener("pageshow", handleWindowFocus);
    document.addEventListener("visibilitychange", handleVisibilityChange);
    return () => {
      window.removeEventListener("focus", handleWindowFocus);
      window.removeEventListener("pageshow", handleWindowFocus);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, [refreshSpotlightState]);

  return {
    hideSpotlight,
    isLoadingVault,
    refreshSpotlightState,
    vaultItems,
  };
}
