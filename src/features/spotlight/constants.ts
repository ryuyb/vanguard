import type { DetailAction } from "@/features/spotlight/types";

export const DETAIL_ACTIONS: readonly DetailAction[] = [
  { label: "复制 用户名", shortcut: ["⌘", "C"], field: "username" },
  { label: "复制 密码", shortcut: ["⌘", "⇧", "C"], field: "password" },
  {
    label: "复制 一次性密码",
    shortcut: ["⌘", "⌥", "C"],
    field: "totp",
    requiresTotp: true,
  },
];

export const COPY_FLASH_DURATION_MS = 180;
