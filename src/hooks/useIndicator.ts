import { useLayoutEffect, useRef, useState, useCallback } from "react";

interface Indicator {
  offset: number;
  size: number;
  ready: boolean;
  /** false during resize — consumer should disable CSS transition */
  animate: boolean;
}

/**
 * Tracks a sliding indicator position for tab/nav components.
 * Recalculates on activeIndex change and container resize.
 * Sets `animate: false` during resize so transitions are skipped.
 */
export function useIndicator<T extends HTMLElement = HTMLElement>(activeIndex: number, axis: "x" | "y" = "x") {
  const containerRef = useRef<T>(null);
  const [indicator, setIndicator] = useState<Indicator>({ offset: 0, size: 0, ready: false, animate: false });
  const isResizingRef = useRef(false);
  const initialRef = useRef(true);

  const measure = useCallback(() => {
    const container = containerRef.current;
    if (!container || activeIndex < 0) return;
    const children = container.querySelectorAll<HTMLElement>(":scope > a, :scope > button");
    const el = children[activeIndex];
    if (!el) return;
    const offset = axis === "x" ? el.offsetLeft : el.offsetTop;
    const size = axis === "x" ? el.offsetWidth : el.offsetHeight;
    const shouldAnimate = !isResizingRef.current && !initialRef.current;
    initialRef.current = false;
    setIndicator({ offset, size, ready: true, animate: shouldAnimate });
  }, [activeIndex, axis]);

  useLayoutEffect(() => {
    isResizingRef.current = false;
    measure();
    const container = containerRef.current;
    if (!container) return;
    const ro = new ResizeObserver(() => {
      isResizingRef.current = true;
      measure();
    });
    ro.observe(container);
    return () => ro.disconnect();
  }, [measure]);

  return { containerRef, indicator };
}
