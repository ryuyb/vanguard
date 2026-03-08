import { commands } from "@/bindings";
import { toErrorText } from "@/features/auth/shared/utils";

export function toUnlockErrorText(error: unknown): string {
  return toErrorText(error, "解锁失败，请稍后重试。");
}

export async function unlockWithMasterPassword(password: string) {
  return commands.vaultUnlock({
    method: {
      type: "masterPassword",
      password,
    },
  });
}

export async function unlockWithPin(pin: string) {
  return commands.vaultUnlock({
    method: {
      type: "pin",
      pin,
    },
  });
}

export async function unlockWithBiometric() {
  return commands.vaultUnlock({
    method: {
      type: "biometric",
    },
  });
}
