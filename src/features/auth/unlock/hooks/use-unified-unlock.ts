import { useCallback, useEffect, useRef, useState } from "react";
import { commands, events, type UnlockStateResponseDto } from "@/bindings";
import { errorHandler } from "@/lib/error-handler";

export type UnifiedUnlockState = UnlockStateResponseDto;

export type UseUnifiedUnlockReturn = {
  /** Current unlock state from backend */
  state: UnifiedUnlockState | null;
  /** Whether state is still loading */
  isLoading: boolean;
  /** Whether vault is fully locked */
  isLocked: boolean;
  /** Whether vault is unlocked (may or may not have valid session) */
  isVaultUnlocked: boolean;
  /** Whether both vault and session are valid */
  isFullyUnlocked: boolean;
  /** Whether an unlock operation is in progress */
  isUnlocking: boolean;
  /** Whether vault is unlocked but session has expired */
  isVaultUnlockedSessionExpired: boolean;
  /** Unlock the vault with master password */
  unlockWithMasterPassword: (password: string) => Promise<boolean>;
  /** Unlock the vault with PIN */
  unlockWithPin: (pin: string) => Promise<boolean>;
  /** Unlock the vault with biometric */
  unlockWithBiometric: () => Promise<boolean>;
  /** Lock the vault */
  lock: () => Promise<boolean>;
  /** Logout and clear all state */
  logout: () => Promise<boolean>;
  /** Refresh the API session */
  refreshSession: () => Promise<boolean>;
  /** Manually refresh state from backend */
  refreshState: () => Promise<UnifiedUnlockState>;
};

const defaultState: UnifiedUnlockState = {
  status: "locked",
  account: null,
  session: null,
  hasKeyMaterial: false,
  unlockMethod: null,
};

/**
 * Hook for unified unlock state management.
 * Provides reactive access to the backend unlock state with automatic subscription.
 */
export function useUnifiedUnlock(): UseUnifiedUnlockReturn {
  const [state, setState] = useState<UnifiedUnlockState | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const unsubscribeRef = useRef<(() => void) | null>(null);

  /**
   * Fetch current state from backend
   * @returns The latest state from backend
   */
  const refreshState = useCallback(async (): Promise<UnifiedUnlockState> => {
    try {
      const result = await commands.getUnlockState({});
      if (result.status === "ok") {
        setState(result.data);
        return result.data;
      }
      errorHandler.handle(result.error);
      setState(defaultState);
      return defaultState;
    } catch (err) {
      errorHandler.handle(err);
      setState(defaultState);
      return defaultState;
    }
  }, []);

  /**
   * Subscribe to unlock state changes
   */
  useEffect(() => {
    let isActive = true;

    const setupSubscription = async () => {
      // Initial state fetch
      await refreshState();
      if (!isActive) return;
      setIsLoading(false);

      // Subscribe to state change events
      const unsubscribe = await events.unlockStateChanged.listen(() => {
        if (!isActive) return;

        // Merge event data with current state or fetch fresh state
        // The event contains partial info, so we fetch full state to be safe
        void refreshState();
      });

      if (isActive) {
        unsubscribeRef.current = unsubscribe;
      } else {
        unsubscribe();
      }
    };

    void setupSubscription();

    return () => {
      isActive = false;
      if (unsubscribeRef.current) {
        unsubscribeRef.current();
        unsubscribeRef.current = null;
      }
    };
  }, [refreshState]);

  /**
   * Derived state properties
   */
  const currentState = state ?? defaultState;
  const isLocked = currentState.status === "locked";
  const isVaultUnlocked =
    currentState.status === "fullyUnlocked" ||
    currentState.status === "vaultUnlockedSessionExpired";
  const isFullyUnlocked = currentState.status === "fullyUnlocked";
  const isUnlocking = currentState.status === "unlocking";
  const isVaultUnlockedSessionExpired =
    currentState.status === "vaultUnlockedSessionExpired";

  /**
   * Unlock with master password
   */
  const unlockWithMasterPassword = useCallback(
    async (password: string): Promise<boolean> => {
      try {
        const result = await commands.vaultUnlock({
          method: { type: "masterPassword", password },
        });
        if (result.status === "ok") {
          await refreshState();
          return true;
        }
        errorHandler.handle(result.error);
        return false;
      } catch (error) {
        errorHandler.handle(error);
        return false;
      }
    },
    [refreshState],
  );

  /**
   * Unlock with PIN
   */
  const unlockWithPin = useCallback(
    async (pin: string): Promise<boolean> => {
      try {
        const result = await commands.vaultUnlock({
          method: { type: "pin", pin },
        });
        if (result.status === "ok") {
          await refreshState();
          return true;
        }
        errorHandler.handle(result.error);
        return false;
      } catch (error) {
        errorHandler.handle(error);
        return false;
      }
    },
    [refreshState],
  );

  /**
   * Unlock with biometric
   */
  const unlockWithBiometric = useCallback(async (): Promise<boolean> => {
    try {
      const result = await commands.vaultUnlock({
        method: { type: "biometric" },
      });
      if (result.status === "ok") {
        await refreshState();
        return true;
      }
      errorHandler.handle(result.error);
      return false;
    } catch (error) {
      errorHandler.handle(error);
      return false;
    }
  }, [refreshState]);

  /**
   * Lock the vault
   */
  const lock = useCallback(async (): Promise<boolean> => {
    try {
      const result = await commands.vaultLock({});
      if (result.status === "ok") {
        await refreshState();
        return true;
      }
      errorHandler.handle(result.error);
      return false;
    } catch (error) {
      errorHandler.handle(error);
      return false;
    }
  }, [refreshState]);

  /**
   * Logout and clear all state
   */
  const logout = useCallback(async (): Promise<boolean> => {
    try {
      const result = await commands.authLogout({});
      if (result.status === "ok") {
        await refreshState();
        return true;
      }
      errorHandler.handle(result.error);
      return false;
    } catch (error) {
      errorHandler.handle(error);
      return false;
    }
  }, [refreshState]);

  /**
   * Refresh the API session
   */
  const refreshSession = useCallback(async (): Promise<boolean> => {
    try {
      const result = await commands.refreshSession({});
      if (result.status === "ok") {
        await refreshState();
        return result.data.success;
      }
      errorHandler.handle(result.error);
      return false;
    } catch (error) {
      errorHandler.handle(error);
      return false;
    }
  }, [refreshState]);

  return {
    state: currentState,
    isLoading,
    isLocked,
    isVaultUnlocked,
    isFullyUnlocked,
    isUnlocking,
    isVaultUnlockedSessionExpired,
    unlockWithMasterPassword,
    unlockWithPin,
    unlockWithBiometric,
    lock,
    logout,
    refreshSession,
    refreshState,
  };
}
