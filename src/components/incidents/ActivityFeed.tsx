import { format } from "date-fns";
import { Clock } from "lucide-react";
import { useEntityAuditLog } from "@/hooks/use-audit";

interface ActivityFeedProps {
  incidentId: string;
}

function formatDate(dateStr: string): string {
  try {
    return format(new Date(dateStr), "MMM d, yyyy HH:mm");
  } catch {
    return dateStr;
  }
}

const ACTION_LABELS: Record<string, string> = {
  created: "Created",
  updated: "Updated",
  deleted: "Moved to trash",
  restored: "Restored",
};

export function ActivityFeed({ incidentId }: ActivityFeedProps) {
  const { data: entries, isLoading } = useEntityAuditLog("incident", incidentId);

  if (isLoading) {
    return (
      <div className="text-sm text-muted-foreground py-4">Loading activity...</div>
    );
  }

  if (!entries || entries.length === 0) {
    return (
      <div className="text-sm text-muted-foreground py-4">
        No activity recorded yet.
      </div>
    );
  }

  return (
    <div className="space-y-0">
      {entries.map((entry, index) => (
        <div key={entry.id} className="flex gap-3 py-2">
          <div className="flex flex-col items-center">
            <div className="flex h-6 w-6 items-center justify-center rounded-full bg-muted">
              <Clock className="h-3 w-3 text-muted-foreground" />
            </div>
            {index < entries.length - 1 && (
              <div className="w-px flex-1 bg-border" />
            )}
          </div>
          <div className="flex-1 pb-2">
            <p className="text-sm">
              <span className="font-medium">
                {ACTION_LABELS[entry.action] ?? entry.action}
              </span>
              {" — "}
              {entry.summary}
            </p>
            <p className="text-xs text-muted-foreground">
              {formatDate(entry.created_at)}
            </p>
          </div>
        </div>
      ))}
    </div>
  );
}
