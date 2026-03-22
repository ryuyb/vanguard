import { commands } from "@/bindings";

export async function resolveNextSessionRoute() {
  const restore = await commands.authRestoreState({});
  if (restore.status === "error" || restore.data.status === "needsLogin") {
    return "/" as const;
  }

  const unlockState = await commands.getUnlockState({});
  if (unlockState.status === "ok" && unlockState.data.status !== "locked") {
    return "/vault" as const;
  }

  return "/unlock" as const;
}
