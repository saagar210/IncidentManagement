import { format } from "date-fns";
import {
  AlertCircle,
  CheckCircle,
  Edit,
  UserPlus,
  ListChecks,
  MessageSquare,
} from "lucide-react";
import { useEntityAuditLog } from "@/hooks/use-audit";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

interface IncidentTimelineStreamProps {
  incidentId: string;
}

function getEventIcon(action: string) {
  switch (action) {
    case "created":
      return <AlertCircle className="h-4 w-4 text-blue-500" />;
    case "resolved":
      return <CheckCircle className="h-4 w-4 text-green-500" />;
    case "status_changed":
    case "updated":
      return <Edit className="h-4 w-4 text-yellow-500" />;
    case "role_assigned":
    case "role_unassigned":
      return <UserPlus className="h-4 w-4 text-purple-500" />;
    case "checklist_item_toggled":
      return <ListChecks className="h-4 w-4 text-teal-500" />;
    default:
      return <MessageSquare className="h-4 w-4 text-muted-foreground" />;
  }
}

function getEventColor(action: string): string {
  switch (action) {
    case "created":
      return "border-blue-500";
    case "resolved":
      return "border-green-500";
    case "status_changed":
    case "updated":
      return "border-yellow-500";
    case "role_assigned":
    case "role_unassigned":
      return "border-purple-500";
    case "checklist_item_toggled":
      return "border-teal-500";
    default:
      return "border-muted-foreground";
  }
}

function formatTimestamp(ts: string): string {
  try {
    return format(new Date(ts), "MMM d, HH:mm");
  } catch {
    return ts;
  }
}

export function IncidentTimelineStream({ incidentId }: IncidentTimelineStreamProps) {
  const { data: entries, isLoading } = useEntityAuditLog("incident", incidentId);

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Timeline</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          {Array.from({ length: 5 }).map((_, i) => (
            <Skeleton key={i} className="h-10 w-full" />
          ))}
        </CardContent>
      </Card>
    );
  }

  const sortedEntries = [...(entries ?? [])].sort(
    (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  );

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">
          Timeline
          {sortedEntries.length > 0 && (
            <span className="ml-2 text-sm font-normal text-muted-foreground">
              {sortedEntries.length} events
            </span>
          )}
        </CardTitle>
      </CardHeader>
      <CardContent>
        {sortedEntries.length === 0 ? (
          <p className="flex h-24 items-center justify-center text-sm text-muted-foreground">
            No timeline events yet
          </p>
        ) : (
          <div className="relative space-y-0">
            {sortedEntries.map((entry, index) => (
              <div key={entry.id} className="flex gap-3 pb-4">
                {/* Timeline connector */}
                <div className="relative flex flex-col items-center">
                  <div
                    className={cn(
                      "flex h-8 w-8 shrink-0 items-center justify-center rounded-full border-2 bg-background",
                      getEventColor(entry.action)
                    )}
                  >
                    {getEventIcon(entry.action)}
                  </div>
                  {index < sortedEntries.length - 1 && (
                    <div className="w-px flex-1 bg-border" />
                  )}
                </div>

                {/* Event content */}
                <div className="flex-1 pt-1">
                  <div className="flex items-center gap-2">
                    <Badge variant="outline" className="text-xs">
                      {entry.action.replace(/_/g, " ")}
                    </Badge>
                    <span className="text-xs text-muted-foreground">
                      {formatTimestamp(entry.created_at)}
                    </span>
                  </div>
                  <p className="mt-0.5 text-sm">{entry.summary}</p>
                  {entry.details && (
                    <p className="mt-0.5 text-xs text-muted-foreground">
                      {entry.details}
                    </p>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
