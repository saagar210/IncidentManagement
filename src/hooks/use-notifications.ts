import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import { useQuarters } from "@/hooks/use-quarters";

export interface AppNotification {
  id: string;
  type: "warning" | "info";
  title: string;
  description: string;
}

export function useNotifications() {
  const { data: overdueCount } = useQuery({
    queryKey: ["overdue-action-items-count"],
    queryFn: () => tauriInvoke<number>("count_overdue_action_items"),
    staleTime: 60000,
  });

  const { data: quarters } = useQuarters();

  const notifications = useMemo(() => {
    const items: AppNotification[] = [];

    // Overdue action items
    if (overdueCount && overdueCount > 0) {
      items.push({
        id: "overdue-actions",
        type: "warning",
        title: "Overdue Action Items",
        description: `${overdueCount} action item${overdueCount > 1 ? "s" : ""} past due date.`,
      });
    }

    // Quarter ending soon (within 14 days)
    if (quarters) {
      const now = new Date();
      const twoWeeks = 14 * 24 * 60 * 60 * 1000;
      for (const q of quarters) {
        const endDate = new Date(q.end_date);
        const diff = endDate.getTime() - now.getTime();
        if (diff > 0 && diff < twoWeeks) {
          items.push({
            id: `quarter-ending-${q.id}`,
            type: "info",
            title: "Quarter Ending Soon",
            description: `${q.label} ends in ${Math.ceil(diff / (24 * 60 * 60 * 1000))} days.`,
          });
        }
      }
    }

    return items;
  }, [overdueCount, quarters]);

  return notifications;
}
