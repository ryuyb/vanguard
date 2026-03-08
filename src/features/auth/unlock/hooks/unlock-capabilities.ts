import type { RestoreAuthStateResponseDto } from "@/bindings";

export type UnlockMethod = "pin" | "masterPassword";

export type UnlockCapabilities = {
  biometricEnabled: boolean;
  biometricSupported: boolean;
  canBiometricUnlock: boolean;
  isVaultUnlocked: boolean;
  pinEnabled: boolean;
  pinSupported: boolean;
  restoreState: RestoreAuthStateResponseDto | null;
  unlockMethod: UnlockMethod;
};

export function createDefaultUnlockCapabilities(): UnlockCapabilities {
  return {
    biometricEnabled: false,
    biometricSupported: false,
    canBiometricUnlock: false,
    isVaultUnlocked: false,
    pinEnabled: false,
    pinSupported: false,
    restoreState: null,
    unlockMethod: "masterPassword",
  };
}
