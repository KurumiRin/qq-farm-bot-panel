import { clsx } from "clsx";
import { ChevronDown } from "lucide-react";
import { type ReactNode, useState } from "react";

interface CardProps {
  children: ReactNode;
  className?: string;
  title?: string;
  subtitle?: string;
  action?: ReactNode;
  /** Enable collapsible mode */
  collapsible?: boolean;
  /** Initial collapsed state (default: false = expanded) */
  defaultCollapsed?: boolean;
}

export function Card({
  children,
  className,
  title,
  subtitle,
  action,
  collapsible,
  defaultCollapsed = false,
}: CardProps) {
  const [collapsed, setCollapsed] = useState(defaultCollapsed);

  return (
    <div
      className={clsx(
        "rounded-card border border-border bg-surface p-3.5 md:p-4",
        className
      )}
    >
      {(title || action) && (
        <div
          className={clsx(
            "flex items-center justify-between",
            collapsible && "cursor-pointer select-none"
          )}
          onClick={collapsible ? () => setCollapsed((v) => !v) : undefined}
        >
          <div className="flex items-center gap-2">
            {collapsible && (
              <ChevronDown
                className={clsx(
                  "size-4 text-on-surface-muted transition-transform duration-200",
                  collapsed && "-rotate-90"
                )}
              />
            )}
            {title && <h3 className="text-sm font-semibold">{title}</h3>}
            {subtitle && <p className="text-xs text-on-surface-muted">{subtitle}</p>}
          </div>
          {action}
        </div>
      )}
      {collapsible ? (
        <div
          className={clsx(
            "grid transition-[grid-template-rows] duration-200 ease-in-out",
            collapsed ? "grid-rows-[0fr]" : "grid-rows-[1fr]"
          )}
        >
          <div className="overflow-hidden">
            <div className="pt-3">{children}</div>
          </div>
        </div>
      ) : (
        (title || action) ? <div className="pt-3">{children}</div> : children
      )}
    </div>
  );
}
