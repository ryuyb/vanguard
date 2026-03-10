import type { VaultCipherItemDto } from "@/bindings";
import type { SpotlightItem } from "@/features/spotlight/types";
import { getCipherIconUrl } from "@/features/vault/utils";

export function toCipherItem(
  cipher: VaultCipherItemDto,
  iconServer?: string | null,
): SpotlightItem {
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
    type: cipher.type,
    iconUrl: getCipherIconUrl(cipher, iconServer),
  };
}
