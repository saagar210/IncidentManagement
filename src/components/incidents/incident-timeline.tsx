import { useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { format, parseISO } from "date-fns";
import { cn } from "@/lib/utils";
import type { Incident } from "@/types/incident";

interface IncidentTimelineProps {
  incidents: Incident[];
}

const SEVERITY_BAR_COLORS: Record<string, string> = {
  Critical: "bg-red-500",
  High: "bg-orange-500",
  Medium: "bg-yellow-500",
  Low: "bg-green-500",
};

function formatDuration(minutes: number | null): string {
  if (minutes === null) return "ongoing";
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (hours < 24) return `${hours}h ${mins}m`;
  const days = Math.floor(hours / 24);
  return `${days}d ${hours % 24}h`;
}

export function IncidentTimeline({ incidents }: IncidentTimelineProps) {
  const navigate = useNavigate();

  const { sortedIncidents, timeRange } = useMemo(() => {
    if (incidents.length === 0) {
      return { sortedIncidents: [], timeRange: { start: 0, end: 1, span: 1 } };
    }

    const sorted = [...incidents].sort(
      (a, b) =>
        new Date(a.started_at).getTime() - new Date(b.started_at).getTime()
    );

    const starts = sorted.map((i) => new Date(i.started_at).getTime());
    const ends = sorted.map((i) => {
      if (i.resolved_at) return new Date(i.resolved_at).getTime();
      return Date.now();
    });

    const start = Math.min(...starts);
    const end = Math.max(...ends);
    const span = Math.max(end - start, 1);

    return { sortedIncidents: sorted, timeRange: { start, end, span } };
  }, [incidents]);

  if (sortedIncidents.length === 0) {
    return (
      <div className="flex h-64 items-center justify-center text-muted-foreground">
        <p>No incidents to show in timeline view.</p>
      </div>
    );
  }

  return (
    <div className="space-y-1 overflow-x-auto">
      {sortedIncidents.map((incident) => {
        const startTime = new Date(incident.started_at).getTime();
        const endTime = incident.resolved_at
          ? new Date(incident.resolved_at).getTime()
          : Date.now();
        const leftPct = ((startTime - timeRange.start) / timeRange.span) * 100;
        const widthPct = Math.max(
          ((endTime - startTime) / timeRange.span) * 100,
          1
        );
        const barColor =
          SEVERITY_BAR_COLORS[incident.severity] ?? "bg-zinc-400";

        return (
          <div
            key={incident.id}
            className="group flex items-center gap-2 rounded px-2 py-1 hover:bg-muted/50 cursor-pointer"
            onClick={() => navigate(`/incidents/${incident.id}`)}
          >
            <div className="w-36 shrink-0 truncate text-xs font-medium">
              {incident.title}
            </div>
            <div className="relative h-5 flex-1 rounded bg-muted/30">
              <div
                className={cn(
                  "absolute top-0.5 h-4 rounded-sm transition-opacity",
                  barColor,
                  "opacity-80 group-hover:opacity-100"
                )}
                style={{
                  left: `${leftPct}%`,
                  width: `${widthPct}%`,
                  minWidth: "4px",
                }}
                title={`${incident.title}\n${format(parseISO(incident.started_at), "MMM d HH:mm")} - ${
                  incident.resolved_at
                    ? format(parseISO(incident.resolved_at), "MMM d HH:mm")
                    : "ongoing"
                }\nDuration: ${formatDuration(incident.duration_minutes)}\nSeverity: ${incident.severity}`}
              />
            </div>
            <div className="w-16 shrink-0 text-right text-[10px] text-muted-foreground">
              {formatDuration(incident.duration_minutes)}
            </div>
          </div>
        );
      })}
      {/* Time axis labels */}
      <div className="flex items-center gap-2 px-2 pt-1">
        <div className="w-36 shrink-0" />
        <div className="flex flex-1 justify-between text-[10px] text-muted-foreground">
          <span>{format(new Date(timeRange.start), "MMM d")}</span>
          <span>
            {format(
              new Date(timeRange.start + timeRange.span / 2),
              "MMM d"
            )}
          </span>
          <span>{format(new Date(timeRange.end), "MMM d")}</span>
        </div>
        <div className="w-16 shrink-0" />
      </div>
      {/* Legend */}
      <div className="flex items-center gap-3 px-2 pt-1">
        <div className="w-36 shrink-0" />
        <div className="flex gap-3 text-[10px] text-muted-foreground">
          {Object.entries(SEVERITY_BAR_COLORS).map(([label, color]) => (
            <div key={label} className="flex items-center gap-1">
              <div className={cn("h-2.5 w-2.5 rounded-sm", color)} />
              {label}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
