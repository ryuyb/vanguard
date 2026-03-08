import { commands } from "@/bindings";
import { toErrorText } from "@/features/auth/shared/utils";

export async function resolveNextSessionRoute() {
  const restore = await commands.authRestoreState({});
  if (restore.status === "error" || restore.data.status === "needsLogin") {
    return "/" as const;
  }

  const unlocked = await commands.vaultIsUnlocked();
  if (unlocked.status === "ok" && unlocked.data) {
    return "/vault" as const;
  }

  return "/unlock" as const;
}

export function toSessionRouteErrorText(error: unknown): string {
  return toErrorText(error, "Unable to resolve session route.");
}
