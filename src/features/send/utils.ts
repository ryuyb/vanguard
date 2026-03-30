import type { SendItemDto } from "@/bindings";

export function isSendExpired(send: SendItemDto): boolean {
  if (!send.expirationDate) return false;
  return new Date(send.expirationDate) < new Date();
}

export function formatSendSubtitle(
  send: SendItemDto,
  t: (key: string, opts?: Record<string, unknown>) => string,
): string {
  if (send.expirationDate) {
    const date = new Date(send.expirationDate).toLocaleDateString();
    return t("send.list.expiresOn", { date });
  }
  if (send.maxAccessCount != null) {
    return t("send.list.viewCount", {
      count: send.accessCount ?? 0,
      max: send.maxAccessCount,
    });
  }
  return t("send.list.noExpiration");
}

export function generateSendLink(
  baseUrl: string,
  accessId?: string | null,
  key?: string | null,
): string {
  if (!accessId || !key) return "";
  const base = baseUrl.replace(/\/$/, "");
  return `${base}/#/send/${accessId}/${key}`;
}
