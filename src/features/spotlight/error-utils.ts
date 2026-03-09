import { toErrorText } from "@/features/auth/shared/utils";

/**
 * @deprecated 使用 errorHandler.handle() 替代
 * 此函数仅用于向后兼容,将在未来版本中移除
 */
export function toSpotlightErrorText(error: unknown): string {
  return toErrorText(error, "Unknown error");
}
