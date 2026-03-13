import { commands } from "@/bindings";
import { toErrorText } from "@/features/auth/shared/utils";
import { appI18n } from "@/i18n";

export function toUnlockErrorText(error: unknown): string {
  return toErrorText(error, appI18n.t("auth.unlock.messages.unlockFailed"));
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
