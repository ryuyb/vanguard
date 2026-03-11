import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import "./main.css";

import { createRouter, RouterProvider } from "@tanstack/react-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Toaster } from "@/components/ui/sonner";
import { resolveSessionRoute } from "@/lib/route-session";
import { routeTree } from "./routeTree.gen";

const router = createRouter({ routeTree });

function shouldNavigateToResolvedRoute(currentPath: string, target: string) {
  return currentPath !== target;
}

async function syncRouteWithSession() {
  try {
    const target = await resolveSessionRoute();
    const currentPath = router.state.location.pathname;
    if (!shouldNavigateToResolvedRoute(currentPath, target)) {
      return;
    }
    await router.navigate({ to: target });
  } catch {
    // ignored: session probe failure should not crash app boot
  }
}

void syncRouteWithSession();
void getCurrentWindow().onFocusChanged(({ payload: focused }) => {
  if (!focused) {
    return;
  }
  void syncRouteWithSession();
});

// Register the router instance for type safety
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <StrictMode>
    <RouterProvider router={router} />
    <Toaster position="top-right" richColors />
  </StrictMode>,
);
