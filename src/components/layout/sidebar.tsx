import { NavLink } from "react-router-dom";
import { BarChart3, AlertTriangle, FileText, Settings, Sun, Moon, Monitor, Trash2, CheckSquare, BookOpen, ArrowRightLeft } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import { cn } from "@/lib/utils";
import { useTheme, type Theme } from "@/hooks/use-theme";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { NotificationCenter } from "@/components/layout/notification-center";

const NAV_ITEMS = [
  { label: "Dashboard", icon: BarChart3, to: "/dashboard", shortcut: "1" },
  { label: "Incidents", icon: AlertTriangle, to: "/incidents", shortcut: "2" },
  { label: "Action Items", icon: CheckSquare, to: "/action-items", shortcut: null },
  { label: "Learnings", icon: BookOpen, to: "/learnings", shortcut: null },
  { label: "Handoff", icon: ArrowRightLeft, to: "/handoff", shortcut: null },
  { label: "Reports", icon: FileText, to: "/reports", shortcut: "3" },
  { label: "Settings", icon: Settings, to: "/settings", shortcut: "4" },
] as const;

const THEME_CYCLE: Theme[] = ["system", "light", "dark"];
const THEME_ICONS = {
  system: Monitor,
  light: Sun,
  dark: Moon,
} as const;
const THEME_LABELS: Record<Theme, string> = {
  system: "System",
  light: "Light",
  dark: "Dark",
};

export function Sidebar() {
  const { theme, setTheme } = useTheme();
  const { data: trashCount } = useQuery({
    queryKey: ["deleted-count"],
    queryFn: () => tauriInvoke<number>("count_deleted_incidents"),
    staleTime: 30000,
  });
  const { data: overdueCount } = useQuery({
    queryKey: ["overdue-action-items-count"],
    queryFn: () => tauriInvoke<number>("count_overdue_action_items"),
    staleTime: 60000,
  });

  const cycleTheme = () => {
    const currentIdx = THEME_CYCLE.indexOf(theme);
    const nextIdx = (currentIdx + 1) % THEME_CYCLE.length;
    setTheme(THEME_CYCLE[nextIdx]);
  };

  const ThemeIcon = THEME_ICONS[theme];

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
                  ? "bg-sidebar-accent text-sidebar-primary"
                  : "text-sidebar-primary/80 hover:bg-sidebar-accent/50 hover:text-sidebar-primary"
              )
            }
          >
            <item.icon className="h-4 w-4" />
            <span className="flex-1">{item.label}</span>
            {item.label === "Action Items" && (overdueCount ?? 0) > 0 && (
              <Badge variant="destructive" className="h-5 min-w-[20px] px-1 text-[10px]">
                {overdueCount}
              </Badge>
            )}
            {item.shortcut && (
              <kbd className="hidden text-[10px] text-sidebar-primary/70 lg:inline">
                {"\u2318"}
                {item.shortcut}
              </kbd>
            )}
          </NavLink>
        ))}
        <NavLink
          to="/trash"
          className={({ isActive }) =>
            cn(
              "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
              isActive
                ? "bg-sidebar-accent text-sidebar-primary"
                : "text-sidebar-primary/80 hover:bg-sidebar-accent/50 hover:text-sidebar-primary"
            )
          }
        >
          <Trash2 className="h-4 w-4" />
          <span className="flex-1">Trash</span>
          {(trashCount ?? 0) > 0 && (
            <Badge variant="secondary" className="h-5 min-w-[20px] px-1 text-[10px]">
              {trashCount}
            </Badge>
          )}
        </NavLink>
      </nav>

      <div className="space-y-1 border-t border-sidebar-border p-3">
        <NotificationCenter />
        <Button
          variant="ghost"
          size="sm"
          onClick={cycleTheme}
          className="w-full justify-start gap-2 text-sidebar-primary/80 hover:text-sidebar-primary"
        >
          <ThemeIcon className="h-4 w-4" />
          <span className="text-xs">{THEME_LABELS[theme]}</span>
        </Button>
      </div>
    </aside>
  );
}
