import { error as logError } from "@tauri-apps/plugin-log";
import { toErrorText } from "@/features/spotlight/utils";

export function logClientError(context: string, error: unknown): void {
  void logError(`[spotlight] ${context}: ${toErrorText(error)}`);
}
