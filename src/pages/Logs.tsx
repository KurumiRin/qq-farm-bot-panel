import { useState, useEffect, useRef, useCallback } from "react";
import { Trash2, ArrowDown } from "lucide-react";
import { Button } from "../components/Button";
import { useTauriEvent } from "../hooks/useTauriEvent";
import type { LogEntry } from "../types";
import * as api from "../api";

const MAX_LOGS = 500;

const LEVEL_STYLES: Record<string, string> = {
  info: "text-blue-600 bg-blue-50",
  warn: "text-yellow-600 bg-yellow-50",
  error: "text-red-600 bg-red-50",
};

function formatTime(timestamp: number): string {
  const d = new Date(timestamp);
  return d.toLocaleTimeString("zh-CN", { hour12: false });
}

export default function LogsPage() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const containerRef = useRef<HTMLDivElement>(null);

  // Load existing logs on mount
  useEffect(() => {
    api.getLogs().then((existing) => setLogs(existing.slice(-MAX_LOGS)));
  }, []);

  // Listen for new log entries pushed from backend
  const handleLogAdded = useCallback((entry: LogEntry) => {
    setLogs((prev) => [...prev, entry].slice(-MAX_LOGS));
  }, []);

  useTauriEvent("log-added", handleLogAdded);

  // Auto-scroll
  useEffect(() => {
    if (autoScroll && containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [logs, autoScroll]);

  const handleScroll = () => {
    if (!containerRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } = containerRef.current;
    setAutoScroll(scrollHeight - scrollTop - clientHeight < 40);
  };

  const handleClear = () => {
    setLogs([]);
  };

  return (
    <div className="flex flex-col h-full gap-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold">日志</h1>
          <p className="text-sm text-on-surface-muted">
            自动化运行记录 · {logs.length} 条
          </p>
        </div>
        <div className="flex items-center gap-2">
          {!autoScroll && (
            <Button
              size="sm"
              variant="ghost"
              icon={<ArrowDown className="size-3.5" />}
              onClick={() => {
                setAutoScroll(true);
                containerRef.current?.scrollTo({
                  top: containerRef.current.scrollHeight,
                  behavior: "smooth",
                });
              }}
            >
              滚到底部
            </Button>
          )}
          <Button
            size="sm"
            variant="ghost"
            icon={<Trash2 className="size-3.5" />}
            onClick={handleClear}
          >
            清空
          </Button>
        </div>
      </div>

      <div
        ref={containerRef}
        onScroll={handleScroll}
        className="flex-1 min-h-0 overflow-y-auto rounded-card border border-border bg-surface font-mono text-xs"
      >
        {logs.length === 0 ? (
          <div className="flex items-center justify-center h-full text-on-surface-muted text-sm">
            暂无日志，等待自动化运行...
          </div>
        ) : (
          <table className="w-full">
            <tbody>
              {logs.map((log, i) => (
                <tr
                  key={`${log.timestamp}-${i}`}
                  className="border-b border-border/50 last:border-0 hover:bg-black/2"
                >
                  <td className="py-1.5 px-3 text-on-surface-muted whitespace-nowrap w-0">
                    {formatTime(log.timestamp)}
                  </td>
                  <td className="py-1.5 px-2 w-0">
                    <span
                      className={`inline-block rounded px-1.5 py-0.5 text-[10px] font-medium uppercase leading-none ${
                        LEVEL_STYLES[log.level] ?? "text-gray-600 bg-gray-50"
                      }`}
                    >
                      {log.level}
                    </span>
                  </td>
                  <td className="py-1.5 px-3">{log.message}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
