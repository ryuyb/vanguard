import { toErrorText } from "@/features/auth/shared/utils";
import { appI18n } from "@/i18n";

/**
 * @deprecated 使用 errorHandler.handle() 替代
 * 此函数仅用于向后兼容,将在未来版本中移除
 */
export function toSpotlightErrorText(error: unknown): string {
  return toErrorText(error, appI18n.t("spotlight.states.unknownError"));
}
