import type { VaultCipherItemDto } from "@/bindings";
import type { SpotlightItem } from "@/features/spotlight/types";

export function toErrorText(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "Unknown error";
}

export function toCipherItem(cipher: VaultCipherItemDto): SpotlightItem {
  const rawName = cipher.name?.trim() ?? "";
  const rawUsername = cipher.username?.trim() ?? "";
  const title = rawName || "Untitled Cipher";
  const subtitle = rawUsername || "Vault item";
  return {
    id: `cipher-${cipher.id}`,
    cipherId: cipher.id,
    title,
    subtitle,
    searchText: `${rawName} ${rawUsername}`.toLowerCase(),
  };
}
