import { useEffect } from "react";
import { events } from "@/bindings";

type UseSendEventsOptions = {
  onSendCreated?: (sendId: string) => void;
  onSendUpdated?: (sendId: string) => void;
  onSendDeleted?: (sendId: string) => void;
};

export function useSendEvents(options?: UseSendEventsOptions) {
  useEffect(() => {
    const unlistenCreated = events.sendCreated.listen((event) => {
      options?.onSendCreated?.(event.payload.sendId);
    });
    const unlistenUpdated = events.sendUpdated.listen((event) => {
      options?.onSendUpdated?.(event.payload.sendId);
    });
    const unlistenDeleted = events.sendDeleted.listen((event) => {
      options?.onSendDeleted?.(event.payload.sendId);
    });

    return () => {
      unlistenCreated.then((fn) => fn());
      unlistenUpdated.then((fn) => fn());
      unlistenDeleted.then((fn) => fn());
    };
  }, [options?.onSendCreated, options?.onSendUpdated, options?.onSendDeleted]);
}
