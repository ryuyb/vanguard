import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/")({
  component: IndexComponent,
});

function IndexComponent() {
  const test = async () => {
    await invoke("login_and_sync", { serverUrl: "", email: "", password: "" });
  };

  return (
    <main>
      <Button onClick={test}>test</Button>
    </main>
  );
}
