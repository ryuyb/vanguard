import { useEffect, useState } from "react";
import { commands } from "@/bindings";

export interface IconState {
  /** Base64-encoded icon data, null if not loaded or no icon available */
  data: string | null;
}

/**
 * Hook for loading cipher icons from the backend cache.
 *
 * This hook loads icons in the background:
 * - Returns immediately with no icon data (shows fallback)
 * - Loads icon asynchronously in background
 * - Updates with icon data when available
 *
 * @param hostname - The hostname to load icon for (e.g., "google.com")
 * @returns Object containing icon data
 */
export function useIcon(hostname: string | null): IconState {
  const [iconData, setIconData] = useState<string | null>(null);

  useEffect(() => {
    // Reset when hostname changes
    setIconData(null);

    if (!hostname) {
      return;
    }

    let mounted = true;

    const loadIcon = async () => {
      try {
        console.log(`[useIcon] Loading icon for: ${hostname}`);
        const result = await commands.getIcon(hostname);

        if (!mounted) {
          return;
        }

        if (result.status === "ok" && result.data) {
          console.log(`[useIcon] Loaded icon for: ${hostname}`);
          setIconData(result.data);
        } else {
          console.log(`[useIcon] No icon for: ${hostname}`);
        }
      } catch (error) {
        console.error(`[useIcon] Error for ${hostname}:`, error);
      }
    };

    void loadIcon();

    return () => {
      mounted = false;
    };
  }, [hostname]);

  return { data: iconData };
}

/**
 * Hook for clearing the icon cache.
 *
 * @returns Function to clear the icon cache
 */
export function useClearIconCache(): () => Promise<void> {
  return async () => {
    const result = await commands.clearIconCache();
    if (result.status === "error") {
      throw new Error("Failed to clear icon cache");
    }
  };
}
