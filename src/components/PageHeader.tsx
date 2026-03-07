import type { ReactNode } from "react";

export interface PageTag {
  label: string;
  value?: string | number;
  cls?: string;
  icon?: ReactNode;
  hidden?: boolean;
}

interface PageHeaderProps {
  title: string;
  tags?: PageTag[];
  actions?: ReactNode;
  /** Extra content rendered at the right side of the tags row */
  tagActions?: ReactNode;
}

export function PageHeader({ title, tags, actions, tagActions }: PageHeaderProps) {
  const visibleTags = tags?.filter((t) => !t.hidden);

  return (
    <div className="sticky top-0 z-10 bg-surface-dim -mx-6 px-6 pt-12 pb-0.5">
      <div className="space-y-1.5">
        <div className="flex items-center justify-between">
          <h1 className="text-xl font-bold">{title}</h1>
          {actions && <div className="flex items-center gap-1.5">{actions}</div>}
        </div>
        {(visibleTags?.length || tagActions) && (
          <div className="flex items-center justify-between gap-2">
            <div className="flex flex-wrap gap-1.5">
              {visibleTags?.map((tag, i) => (
                <span
                  key={i}
                  className={`inline-flex items-center gap-0.5 rounded-full px-2 py-0.5 text-[11px] font-medium ${tag.cls ?? "bg-surface-bright text-on-surface-muted"}`}
                >
                  {tag.icon}
                  {tag.label}{tag.value != null && tag.value !== "" && <>{tag.label ? " " : ""}<span className="font-semibold">{tag.value}</span></>}
                </span>
              ))}
            </div>
            {tagActions}
          </div>
        )}
      </div>
      <div className="absolute bottom-0 left-0 right-0 -mb-6 h-6 pointer-events-none bg-linear-to-b from-surface-dim to-transparent" />
    </div>
  );
}
