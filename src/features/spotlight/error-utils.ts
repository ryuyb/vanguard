import { toErrorText } from "@/features/auth/shared/utils";

export function toSpotlightErrorText(error: unknown): string {
  return toErrorText(error, "Unknown error");
}
