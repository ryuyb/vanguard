import { error as logError } from "@tauri-apps/plugin-log";
import { toSpotlightErrorText } from "@/features/spotlight/error-utils";

export function logClientError(context: string, error: unknown): void {
  void logError(`[spotlight] ${context}: ${toSpotlightErrorText(error)}`);
}
