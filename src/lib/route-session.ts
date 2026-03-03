import { commands } from "@/bindings";

export type SessionRoute = "/" | "/unlock" | "/vault";

export async function resolveSessionRoute(): Promise<SessionRoute> {
  const restore = await commands.authRestoreState({});
  if (restore.status === "error" || restore.data.status === "needsLogin") {
    return "/";
  }

  const unlocked = await commands.vaultIsUnlocked();
  if (unlocked.status === "ok" && unlocked.data) {
    return "/vault";
  }

  return "/unlock";
}
