import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";

import { LoginForm } from "@/components/login-form";
import { RegisterForm } from "@/components/register-form";

export const Route = createFileRoute("/auth")({
  component: AuthPage,
});

function AuthPage() {
  const [mode, setMode] = useState<"login" | "register">("login");

  return (
    <main className="bg-background text-foreground flex min-h-screen items-center justify-center p-6">
      {mode === "login" ? (
        <LoginForm onRegisterClick={() => setMode("register")} />
      ) : (
        <RegisterForm onLoginClick={() => setMode("login")} />
      )}
    </main>
  );
}
