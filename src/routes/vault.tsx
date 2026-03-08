import { createFileRoute, redirect, useNavigate } from "@tanstack/react-router";
import { useCallback } from "react";
import { VaultPage } from "@/features/vault";
import { resolveSessionRoute } from "@/lib/route-session";

export const Route = createFileRoute("/vault")({
  beforeLoad: async () => {
    const target = await resolveSessionRoute();
    if (target !== "/vault") {
      throw redirect({ to: target });
    }
  },
  component: VaultRoute,
});

function VaultRoute() {
  const navigate = useNavigate({ from: "/vault" });
  const navigateTo = useCallback(
    async (to: "/" | "/unlock" | "/vault") => {
      await navigate({ to });
    },
    [navigate],
  );

  return <VaultPage navigateTo={navigateTo} />;
}
