import { useMemo } from "react";
import { useQuarters } from "@/hooks/use-quarters";
import { useNotificationSummary } from "@/hooks/use-audit";

export interface AppNotification {
  id: string;
  type: "warning" | "info" | "error";
  title: string;
  description: string;
}

export function useNotifications() {
  const { data: summary } = useNotificationSummary();
  const { data: quarters } = useQuarters();

  const notifications = useMemo(() => {
    const items: AppNotification[] = [];

    // SLA breaches (highest priority)
    if (summary && summary.sla_breaches > 0) {
      items.push({
        id: "sla-breaches",
        type: "error",
        title: "SLA Breaches",
        description: `${summary.sla_breaches} active incident${summary.sla_breaches > 1 ? "s" : ""} have breached SLA targets.`,
      });
    }

    // Overdue action items
    if (summary && summary.overdue_action_items > 0) {
      items.push({
        id: "overdue-actions",
        type: "warning",
        title: "Overdue Action Items",
        description: `${summary.overdue_action_items} action item${summary.overdue_action_items > 1 ? "s" : ""} past due date.`,
      });
    }

    // Active incidents
    if (summary && summary.active_incidents > 0) {
      items.push({
        id: "active-incidents",
        type: "info",
        title: "Active Incidents",
        description: `${summary.active_incidents} incident${summary.active_incidents > 1 ? "s" : ""} currently active.`,
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
  }, [summary, quarters]);

  return notifications;
}
