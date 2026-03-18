import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useCallback } from "react";
import { RegisterPage } from "@/features/auth/register";

export const Route = createFileRoute("/register")({
  component: Register,
});

function Register() {
  const navigate = useNavigate({ from: "/register" });
  const navigateToLogin = useCallback(async () => {
    await navigate({ to: "/" });
  }, [navigate]);

  return <RegisterPage navigateToLogin={navigateToLogin} />;
}
