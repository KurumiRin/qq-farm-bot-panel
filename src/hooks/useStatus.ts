import { useState, useEffect, useCallback } from "react";
import type { StatusResponse } from "../types";
import * as api from "../api";
import { useTauriEvent } from "./useTauriEvent";

export function useStatus() {
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Initial fetch
  useEffect(() => {
    api
      .getStatus()
      .then((s) => {
        setStatus(s);
        setError(null);
      })
      .catch((e) => setError(String(e)));
  }, []);

  // Listen for backend push events
  const handleStatusChanged = useCallback((payload: StatusResponse) => {
    setStatus(payload);
    setError(null);
  }, []);

  useTauriEvent("status-changed", handleStatusChanged);

  return { status, error };
}
