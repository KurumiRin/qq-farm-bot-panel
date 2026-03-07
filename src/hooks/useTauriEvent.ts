import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";

export function useTauriEvent<T>(event: string, handler: (payload: T) => void, debounceMs?: number) {
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const lastPayloadRef = useRef<T | undefined>(undefined);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    listen<T>(event, (e) => {
      if (cancelled) return;
      if (!debounceMs) {
        handler(e.payload);
        return;
      }
      // Debounce: keep latest payload, delay execution
      lastPayloadRef.current = e.payload;
      clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => {
        if (!cancelled && lastPayloadRef.current !== undefined) {
          handler(lastPayloadRef.current);
        }
      }, debounceMs);
    }).then((f) => {
      if (cancelled) f();
      else unlisten = f;
    });

    return () => {
      cancelled = true;
      clearTimeout(timerRef.current);
      unlisten?.();
    };
  }, [event, handler, debounceMs]);
}
