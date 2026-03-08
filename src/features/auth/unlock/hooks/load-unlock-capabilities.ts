import { commands } from "@/bindings";
import {
  createDefaultUnlockCapabilities,
  type UnlockCapabilities,
} from "./unlock-capabilities";

export async function loadUnlockCapabilities(): Promise<UnlockCapabilities> {
  const result = await commands.authRestoreState({});
  if (result.status === "error") {
    return createDefaultUnlockCapabilities();
  }

  const restoreState = result.data;
  let isVaultUnlocked = false;
  if (restoreState.status !== "needsLogin") {
    const unlockedResult = await commands.vaultIsUnlocked();
    isVaultUnlocked = unlockedResult.status === "ok" && unlockedResult.data;
  }

  const [biometricStatus, pinStatus] = await Promise.all([
    commands.vaultGetBiometricStatus(),
    commands.vaultGetPinStatus(),
  ]);

  const biometricSupported =
    biometricStatus.status === "ok" && biometricStatus.data.supported;
  const biometricEnabled =
    biometricStatus.status === "ok" && biometricStatus.data.enabled;
  const pinSupported = pinStatus.status === "ok" && pinStatus.data.supported;
  const pinEnabled = pinStatus.status === "ok" && pinStatus.data.enabled;

  let canBiometricUnlock = false;
  if (!isVaultUnlocked && biometricSupported && biometricEnabled) {
    const biometricUnlockResult = await commands.vaultCanUnlockWithBiometric();
    canBiometricUnlock =
      biometricUnlockResult.status === "ok" && biometricUnlockResult.data;
  }

  return {
    biometricEnabled,
    biometricSupported,
    canBiometricUnlock,
    isVaultUnlocked,
    pinEnabled,
    pinSupported,
    restoreState,
    unlockMethod: pinEnabled ? "pin" : "masterPassword",
  };
}
