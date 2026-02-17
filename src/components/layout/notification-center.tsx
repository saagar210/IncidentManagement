import { useState, useRef, useEffect } from "react";
import { Bell, AlertTriangle, Info, XCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useNotifications, type AppNotification } from "@/hooks/use-notifications";

function NotificationIcon({ type }: { type: AppNotification["type"] }) {
  if (type === "error") return <XCircle className="h-4 w-4 text-red-500" />;
  if (type === "warning") return <AlertTriangle className="h-4 w-4 text-amber-500" />;
  return <Info className="h-4 w-4 text-blue-500" />;
}

export function NotificationCenter() {
  const notifications = useNotifications();
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const count = notifications.length;

  return (
    <div ref={containerRef} className="relative">
      <Button
        variant="ghost"
        size="sm"
        onClick={() => setOpen((prev) => !prev)}
        className="relative w-full justify-start gap-2 text-sidebar-primary/80 hover:text-sidebar-primary"
      >
        <Bell className="h-4 w-4" />
        <span className="text-xs">Notifications</span>
        {count > 0 && (
          <Badge
            variant="destructive"
            className="ml-auto h-5 min-w-[20px] px-1 text-[10px]"
          >
            {count}
          </Badge>
        )}
      </Button>

      {open && (
        <div className="absolute bottom-full left-0 mb-1 w-72 rounded-md border bg-popover shadow-lg z-50">
          <div className="p-3 border-b">
            <p className="text-sm font-medium">Notifications</p>
          </div>
          {notifications.length === 0 ? (
            <div className="p-4 text-center text-sm text-muted-foreground">
              All clear — no notifications.
            </div>
          ) : (
            <div className="max-h-64 overflow-y-auto">
              {notifications.map((n) => (
                <div
                  key={n.id}
                  className="flex items-start gap-3 border-b last:border-0 px-3 py-2.5"
                >
                  <NotificationIcon type={n.type} />
                  <div className="min-w-0">
                    <p className="text-sm font-medium">{n.title}</p>
                    <p className="text-xs text-muted-foreground">
                      {n.description}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
