import type { VaultCipherItemDto } from "@/bindings";
import type { SpotlightItem } from "@/features/spotlight/types";
import { toCipherIconHost } from "@/features/vault/utils";
import { appI18n } from "@/i18n";

export function toCipherItem(cipher: VaultCipherItemDto): SpotlightItem {
  const rawName = cipher.name?.trim() ?? "";
  const rawUsername = cipher.username?.trim() ?? "";
  const title = rawName || appI18n.t("spotlight.items.untitledCipher");
  const subtitle = rawUsername || appI18n.t("spotlight.items.defaultSubtitle");

  // Extract hostname for icon lookup
  const firstUri = cipher.uris?.[0] ?? null;
  const iconHostname = firstUri ? toCipherIconHost(firstUri) : null;

  return {
    id: `cipher-${cipher.id}`,
    cipherId: cipher.id,
    title,
    subtitle,
    searchText: `${rawName} ${rawUsername}`.toLowerCase(),
    type: cipher.type ?? 0,
    iconHostname,
  };
}
