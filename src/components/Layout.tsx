import { NavLink, Outlet } from "react-router-dom";
import { clsx } from "clsx";
import {
  LayoutDashboard,
  Sprout,
  Users,
  Package,
  ListTodo,
  Settings,
  LogOut,
  Wifi,
  WifiOff,
  Menu,
  X,
} from "lucide-react";
import { useState, useCallback } from "react";
import { useStatus } from "../hooks/useStatus";
import * as api from "../api";

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/farm", icon: Sprout, label: "Farm" },
  { to: "/friends", icon: Users, label: "Friends" },
  { to: "/inventory", icon: Package, label: "Inventory" },
  { to: "/tasks", icon: ListTodo, label: "Tasks" },
  { to: "/settings", icon: Settings, label: "Settings" },
] as const;

function ConnectionBadge({ connected }: { connected: boolean }) {
  return (
    <div
      className={clsx(
        "flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium",
        connected
          ? "bg-primary-100 text-primary-700"
          : "bg-red-100 text-red-700"
      )}
    >
      {connected ? <Wifi className="size-3" /> : <WifiOff className="size-3" />}
      {connected ? "Online" : "Offline"}
    </div>
  );
}

export default function Layout() {
  const [mobileOpen, setMobileOpen] = useState(false);
  const { status } = useStatus();

  const isConnected =
    status?.connection === "LoggedIn" || status?.connection === "Connected";

  const handleDisconnect = useCallback(async () => {
    try {
      await api.disconnect();
    } catch (e) {
      console.error("Disconnect failed:", e);
    }
  }, []);

  const closeMobile = () => setMobileOpen(false);

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Desktop Sidebar */}
      <aside className="hidden md:flex md:w-60 flex-col border-r border-border bg-surface">
        <div className="flex items-center gap-3 px-5 py-4 border-b border-border">
          <Sprout className="size-7 text-primary-500" />
          <span className="text-lg font-semibold">Farm Pilot</span>
        </div>

        {status?.user && status.user.gid !== 0 && (
          <div className="px-5 py-3 border-b border-border">
            <p className="text-sm font-medium truncate">{status.user.name}</p>
            <p className="text-xs text-on-surface-muted">
              Lv.{status.user.level} | Gold: {status.user.gold.toLocaleString()}
            </p>
          </div>
        )}

        <nav className="flex-1 px-3 py-2 space-y-0.5 overflow-y-auto">
          {navItems.map(({ to, icon: Icon, label }) => (
            <NavLink
              key={to}
              to={to}
              end={to === "/"}
              className={({ isActive }) =>
                clsx(
                  "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-primary-50 text-primary-700"
                    : "text-on-surface-muted hover:bg-surface-bright hover:text-on-surface"
                )
              }
            >
              <Icon className="size-4" />
              {label}
            </NavLink>
          ))}
        </nav>

        <div className="px-3 py-3 border-t border-border space-y-2">
          <ConnectionBadge connected={isConnected} />
          {isConnected && (
            <button
              onClick={handleDisconnect}
              className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-red-600 hover:bg-red-50 transition-colors"
            >
              <LogOut className="size-4" />
              Disconnect
            </button>
          )}
        </div>
      </aside>

      {/* Mobile overlay */}
      {mobileOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/30 md:hidden"
          onClick={closeMobile}
        />
      )}

      {/* Mobile sidebar */}
      <aside
        className={clsx(
          "fixed inset-y-0 left-0 z-50 w-64 bg-surface border-r border-border transition-transform duration-200 md:hidden",
          mobileOpen ? "translate-x-0" : "-translate-x-full"
        )}
      >
        <div className="flex items-center justify-between px-5 py-4 border-b border-border">
          <div className="flex items-center gap-3">
            <Sprout className="size-7 text-primary-500" />
            <span className="text-lg font-semibold">Farm Pilot</span>
          </div>
          <button onClick={closeMobile}>
            <X className="size-5 text-on-surface-muted" />
          </button>
        </div>

        {status?.user && status.user.gid !== 0 && (
          <div className="px-5 py-3 border-b border-border">
            <p className="text-sm font-medium truncate">{status.user.name}</p>
            <p className="text-xs text-on-surface-muted">
              Lv.{status.user.level} | Gold: {status.user.gold.toLocaleString()}
            </p>
          </div>
        )}

        <nav className="px-3 py-2 space-y-0.5">
          {navItems.map(({ to, icon: Icon, label }) => (
            <NavLink
              key={to}
              to={to}
              end={to === "/"}
              onClick={closeMobile}
              className={({ isActive }) =>
                clsx(
                  "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-primary-50 text-primary-700"
                    : "text-on-surface-muted hover:bg-surface-bright hover:text-on-surface"
                )
              }
            >
              <Icon className="size-4" />
              {label}
            </NavLink>
          ))}
        </nav>

        <div className="absolute bottom-0 left-0 right-0 px-3 py-3 border-t border-border space-y-2">
          <ConnectionBadge connected={isConnected} />
          {isConnected && (
            <button
              onClick={handleDisconnect}
              className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-red-600 hover:bg-red-50 transition-colors"
            >
              <LogOut className="size-4" />
              Disconnect
            </button>
          )}
        </div>
      </aside>

      {/* Main content */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Mobile header */}
        <header className="flex items-center gap-3 px-4 py-3 border-b border-border bg-surface md:hidden">
          <button onClick={() => setMobileOpen(true)}>
            <Menu className="size-5 text-on-surface" />
          </button>
          <span className="font-semibold">Farm Pilot</span>
          <div className="ml-auto">
            <ConnectionBadge connected={isConnected} />
          </div>
        </header>

        <main className="flex-1 overflow-y-auto p-4 md:p-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
