import { commands } from "@/bindings";

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
