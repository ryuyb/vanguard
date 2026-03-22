import { createFileRoute, redirect, useNavigate } from "@tanstack/react-router";
import { useCallback } from "react";
import { UnlockPage } from "@/features/auth/unlock";
import { resolveNextSessionRoute } from "@/lib/route-session-utils";

export const Route = createFileRoute("/unlock")({
  beforeLoad: async () => {
    const target = await resolveNextSessionRoute();
    if (target !== "/unlock") {
      throw redirect({ to: target });
    }
  },
  component: UnlockRoute,
});

function UnlockRoute() {
  const navigate = useNavigate({ from: "/unlock" });
  const navigateToVault = useCallback(async () => {
    await navigate({ to: "/vault" });
  }, [navigate]);
  const navigateToHome = useCallback(async () => {
    await navigate({ to: "/" });
  }, [navigate]);

  return (
    <UnlockPage
      navigateToHome={navigateToHome}
      navigateToVault={navigateToVault}
    />
  );
}
