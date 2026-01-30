import { createFileRoute } from "@tanstack/react-router";
import { LoginForm } from "@/components/login-form";

export const Route = createFileRoute("/")({
  component: IndexComponent,
});

function IndexComponent() {
  return (
    <main className="bg-background text-foreground flex min-h-screen items-center justify-center p-6">
      <LoginForm />
    </main>
  );
}
