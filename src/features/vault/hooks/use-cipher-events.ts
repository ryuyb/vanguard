import { useEffect } from "react";
import { events } from "@/bindings";

type UseCipherEventsOptions = {
  onCipherCreated?: (cipherId: string) => void;
  onCipherUpdated?: (cipherId: string) => void;
  onCipherDeleted?: (cipherId: string) => void;
};

export function useCipherEvents(options?: UseCipherEventsOptions) {
  useEffect(() => {
    const unlistenCreated = events.cipherCreated.listen((event) => {
      console.log(`Cipher created: ${event.payload.cipherId}`);
      options?.onCipherCreated?.(event.payload.cipherId);
    });

    const unlistenUpdated = events.cipherUpdated.listen((event) => {
      console.log(`Cipher updated: ${event.payload.cipherId}`);
      options?.onCipherUpdated?.(event.payload.cipherId);
    });

    const unlistenDeleted = events.cipherDeleted.listen((event) => {
      console.log(`Cipher deleted: ${event.payload.cipherId}`);
      options?.onCipherDeleted?.(event.payload.cipherId);
    });

    return () => {
      unlistenCreated.then((fn) => fn());
      unlistenUpdated.then((fn) => fn());
      unlistenDeleted.then((fn) => fn());
    };
  }, [
    options?.onCipherCreated,
    options?.onCipherUpdated,
    options?.onCipherDeleted,
  ]);
}
