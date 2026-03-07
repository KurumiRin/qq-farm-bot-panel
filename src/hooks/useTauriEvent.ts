import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

export function useTauriEvent<T>(event: string, handler: (payload: T) => void) {
  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    listen<T>(event, (e) => {
      if (!cancelled) handler(e.payload);
    }).then((f) => {
      if (cancelled) f();
      else unlisten = f;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [event, handler]);
}
