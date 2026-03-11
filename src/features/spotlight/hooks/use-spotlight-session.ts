import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useState } from "react";
import { commands } from "@/bindings";
import type { SpotlightItem } from "@/features/spotlight/types";
import { toCipherItem } from "@/features/spotlight/utils";
import { resolveSessionRoute } from "@/lib/route-session";
import { errorHandler } from "@/lib/error-handler";

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
      errorHandler.handle(error);
    }
  }, []);

  const openMainWindow = useCallback(async () => {
    try {
      const result = await commands.desktopOpenMainWindow();
      if (result.status === "error") {
        errorHandler.handle(result.error);
      }
    } catch (error) {
      errorHandler.handle(error);
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
      errorHandler.handle(error);
      await hideSpotlight();
      return false;
    }
  }, [hideSpotlight, openMainWindow]);

  const loadVaultItems = useCallback(async () => {
    setIsLoadingVault(true);

    try {
      const viewData = await commands.vaultGetViewData();
      if (viewData.status === "error") {
        errorHandler.handle(viewData.error);
        setVaultItems([]);
        return;
      }

      const iconServerResult = await commands.vaultGetIconServer();
      const iconServer = iconServerResult.status === "ok" ? iconServerResult.data : null;
      const ciphers = viewData.data.ciphers.map((cipher) =>
        toCipherItem(cipher, iconServer),
      );
      setVaultItems(ciphers);
    } catch (error) {
      errorHandler.handle(error);
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
        errorHandler.handle(error);
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
