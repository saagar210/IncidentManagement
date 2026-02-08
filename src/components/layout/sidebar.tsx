import { NavLink } from "react-router-dom";
import { BarChart3, AlertTriangle, FileText, Settings } from "lucide-react";
import { cn } from "@/lib/utils";

const NAV_ITEMS = [
  { label: "Dashboard", icon: BarChart3, to: "/dashboard", shortcut: "1" },
  { label: "Incidents", icon: AlertTriangle, to: "/incidents", shortcut: "2" },
  { label: "Reports", icon: FileText, to: "/reports", shortcut: "3" },
  { label: "Settings", icon: Settings, to: "/settings", shortcut: "4" },
] as const;

export function Sidebar() {
  return (
    <aside className="flex h-full w-60 flex-col border-r border-sidebar-border bg-sidebar-background">
      <div className="flex h-14 items-center gap-2 border-b border-sidebar-border px-4">
        <AlertTriangle className="h-5 w-5 text-sidebar-primary" />
        <span className="text-sm font-semibold text-sidebar-foreground">
          Incident Manager
        </span>
      </div>

      <nav className="flex-1 space-y-1 p-2">
        {NAV_ITEMS.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              cn(
                "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                isActive
                  ? "bg-sidebar-accent text-sidebar-accent-foreground"
                  : "text-sidebar-foreground/70 hover:bg-sidebar-accent/50 hover:text-sidebar-foreground"
              )
            }
          >
            <item.icon className="h-4 w-4" />
            <span className="flex-1">{item.label}</span>
            <kbd className="hidden text-[10px] text-sidebar-foreground/40 lg:inline">
              {"\u2318"}
              {item.shortcut}
            </kbd>
          </NavLink>
        ))}
      </nav>

      <div className="border-t border-sidebar-border p-4">
        <p className="text-xs text-sidebar-foreground/50">Theme toggle (coming soon)</p>
      </div>
    </aside>
  );
}
