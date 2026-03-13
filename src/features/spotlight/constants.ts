import type { DetailAction } from "@/features/spotlight/types";
import { appI18n } from "@/i18n";

export const getDetailActions = (): readonly DetailAction[] => [
  {
    label: appI18n.t("spotlight.actions.copyUsername"),
    shortcut: ["⌘", "C"],
    field: "username",
  },
  {
    label: appI18n.t("spotlight.actions.copyPassword"),
    shortcut: ["⌘", "⇧", "C"],
    field: "password",
  },
  {
    label: appI18n.t("spotlight.actions.copyTotp"),
    shortcut: ["⌘", "⌥", "C"],
    field: "totp",
    requiresTotp: true,
  },
];

export const COPY_FLASH_DURATION_MS = 180;
