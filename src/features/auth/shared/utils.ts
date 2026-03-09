/**
 * @deprecated 使用 errorHandler.handle() 替代
 * 此函数仅用于向后兼容,将在未来版本中移除
 */
export function toErrorText(error: unknown, fallback: string): string {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return fallback;
}
