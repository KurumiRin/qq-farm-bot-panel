import { useState, useEffect, useCallback, useRef } from "react";
import type { StatusResponse } from "../types";
import * as api from "../api";

export function useStatus(polling = true, intervalMs = 2000) {
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval>>(undefined);

  const refresh = useCallback(async () => {
    try {
      const s = await api.getStatus();
      setStatus(s);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    refresh();
    if (polling) {
      timerRef.current = setInterval(refresh, intervalMs);
      return () => clearInterval(timerRef.current);
    }
  }, [polling, intervalMs, refresh]);

  return { status, error, refresh };
}
