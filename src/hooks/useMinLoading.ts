import { useCallback, useRef, useState } from "react";

const MIN_MS = 500;

/**
 * Like useState<boolean> but ensures `true` lasts at least MIN_MS
 * so spin animations complete smoothly.
 */
export function useMinLoading(initial = false): [boolean, (v: boolean) => void] {
  const [loading, setLoading] = useState(initial);
  const startRef = useRef(0);

  const set = useCallback((v: boolean) => {
    if (v) {
      startRef.current = Date.now();
      setLoading(true);
    } else {
      const elapsed = Date.now() - startRef.current;
      const remaining = MIN_MS - elapsed;
      if (remaining > 0) {
        setTimeout(() => setLoading(false), remaining);
      } else {
        setLoading(false);
      }
    }
  }, []);

  return [loading, set];
}
