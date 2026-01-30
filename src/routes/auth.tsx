import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";

import { LoginForm } from "@/components/auth/login-form";
import { RegisterForm } from "@/components/auth/register-form";
import { WelcomeBack } from "@/components/auth/welcome-back";

export const Route = createFileRoute("/auth")({
  component: AuthPage,
});

function AuthPage() {
  const [loginEmail, setLoginEmail] = useState<string>("");
  const [mode, setMode] = useState<"login" | "register" | "welcome-back">(
    "login",
  );

  return (
    <main className="bg-background text-foreground flex min-h-screen items-center justify-center p-6">
      {mode === "login" ? (
        <LoginForm
          onContinue={({ email }) => {
            setLoginEmail(email);
            setMode("welcome-back");
          }}
          onRegisterClick={() => setMode("register")}
        />
      ) : null}
      {mode === "register" ? (
        <RegisterForm onLoginClick={() => setMode("login")} />
      ) : null}
      {mode === "welcome-back" ? (
        <WelcomeBack
          email={loginEmail}
          onBack={() => setMode("login")}
          onRegisterClick={() => setMode("register")}
        />
      ) : null}
    </main>
  );
}
