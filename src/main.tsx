import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import "./main.css";

import { TanStackDevtools } from "@tanstack/react-devtools";
import { formDevtoolsPlugin } from "@tanstack/react-form-devtools";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createRouter, RouterProvider } from "@tanstack/react-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { AppLocaleProvider, initializeAppI18n } from "@/i18n";
import { resolveNextSessionRoute } from "@/lib/route-session-utils";
import { routeTree } from "./routeTree.gen";

const router = createRouter({ routeTree });

const PRE_AUTH_ROUTES = new Set(["/", "/register"]);

function shouldNavigateToResolvedRoute(currentPath: string, target: string) {
  if (target === "/" && PRE_AUTH_ROUTES.has(currentPath)) {
    return false;
  }
  return currentPath !== target;
}

async function syncRouteWithSession() {
  try {
    const target = await resolveNextSessionRoute();
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

async function bootstrap() {
  await initializeAppI18n();

  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <StrictMode>
      <AppLocaleProvider>
        <QueryClientProvider client={queryClient}>
          <TooltipProvider>
            <RouterProvider router={router} />
            <Toaster position="top-right" richColors />
          </TooltipProvider>
        </QueryClientProvider>
      </AppLocaleProvider>
      <TanStackDevtools plugins={[formDevtoolsPlugin()]} />
    </StrictMode>,
  );
}

void bootstrap();
