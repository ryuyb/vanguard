import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import "./main.css";

import { TanStackDevtools } from "@tanstack/react-devtools";
import { formDevtoolsPlugin } from "@tanstack/react-form-devtools";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
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

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
      <Toaster position="top-right" richColors />
    </QueryClientProvider>
    <TanStackDevtools plugins={[formDevtoolsPlugin()]} />
  </StrictMode>,
);
