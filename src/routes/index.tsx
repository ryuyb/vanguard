import { createFileRoute, redirect, useNavigate } from "@tanstack/react-router";
import { useCallback } from "react";
import { LoginPage } from "@/features/auth/login";
import { resolveNextSessionRoute } from "@/lib/route-session-utils";

export const Route = createFileRoute("/")({
  beforeLoad: async () => {
    const target = await resolveNextSessionRoute();
    if (target !== "/") {
      throw redirect({ to: target });
    }
  },
  component: Index,
});

function Index() {
  const navigate = useNavigate({ from: "/" });
  const navigateToVault = useCallback(async () => {
    await navigate({ to: "/vault" });
  }, [navigate]);
  const navigateToRegister = useCallback(async () => {
    await navigate({ to: "/register" });
  }, [navigate]);

  return (
    <LoginPage
      navigateToVault={navigateToVault}
      navigateToRegister={navigateToRegister}
    />
  );
}
