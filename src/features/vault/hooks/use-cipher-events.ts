import { useEffect } from "react";
import { events } from "@/bindings";

type UseCipherEventsOptions = {
  onCipherCreated?: (cipherId: string) => void;
  onCipherUpdated?: (cipherId: string) => void;
  onCipherDeleted?: (cipherId: string) => void;
};

export function useCipherEvents(options?: UseCipherEventsOptions) {
  useEffect(() => {
    let cleanupStarted = false;
    const unlistenFns: Array<() => void> = [];

    const setupListeners = async () => {
      const [unlistenCreated, unlistenUpdated, unlistenDeleted] =
        await Promise.all([
          events.cipherCreated.listen((event) => {
            options?.onCipherCreated?.(event.payload.cipherId);
          }),
          events.cipherUpdated.listen((event) => {
            options?.onCipherUpdated?.(event.payload.cipherId);
          }),
          events.cipherDeleted.listen((event) => {
            options?.onCipherDeleted?.(event.payload.cipherId);
          }),
        ]);

      unlistenFns.push(unlistenCreated, unlistenUpdated, unlistenDeleted);

      // If cleanup started during async setup, immediately unlisten
      if (cleanupStarted) {
        for (const fn of unlistenFns) {
          fn();
        }
      }
    };

    setupListeners();

    return () => {
      cleanupStarted = true;
      for (const fn of unlistenFns) {
        fn();
      }
    };
  }, [
    options?.onCipherCreated,
    options?.onCipherUpdated,
    options?.onCipherDeleted,
  ]);
}
