import { resolveNextSessionRoute } from "@/lib/route-session-utils";

export type SessionRoute = "/" | "/unlock" | "/vault";

export async function resolveSessionRoute(): Promise<SessionRoute> {
  return resolveNextSessionRoute();
}
