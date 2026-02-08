import { useSlaStatus } from "@/hooks/use-sla";
import { Badge } from "@/components/ui/badge";
import { Clock, AlertTriangle, CheckCircle } from "lucide-react";

interface SlaStatusBadgeProps {
  incidentId: string;
  compact?: boolean;
}

function formatMinutes(minutes: number): string {
  if (minutes < 0) minutes = 0;
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (hours < 24) return `${hours}h ${mins}m`;
  const days = Math.floor(hours / 24);
  const remainingHours = hours % 24;
  return `${days}d ${remainingHours}h`;
}

function SlaIndicator({
  label,
  elapsed,
  target,
  breached,
}: {
  label: string;
  elapsed: number | null;
  target: number | null;
  breached: boolean;
}) {
  if (target === null) return null;

  const remaining = target - (elapsed ?? 0);
  const percentage = elapsed !== null ? Math.min((elapsed / target) * 100, 100) : 0;
  const isWarning = !breached && percentage >= 75;

  return (
    <div className="flex items-center gap-1.5">
      {breached ? (
        <AlertTriangle className="h-3.5 w-3.5 text-red-500" />
      ) : isWarning ? (
        <Clock className="h-3.5 w-3.5 text-yellow-500" />
      ) : (
        <CheckCircle className="h-3.5 w-3.5 text-green-500" />
      )}
      <span className="text-xs">
        {label}:{" "}
        {breached ? (
          <span className="font-medium text-red-500">
            Breached ({formatMinutes(elapsed ?? 0)} / {formatMinutes(target)})
          </span>
        ) : (
          <span
            className={
              isWarning
                ? "font-medium text-yellow-600"
                : "text-muted-foreground"
            }
          >
            {formatMinutes(remaining)} remaining
          </span>
        )}
      </span>
    </div>
  );
}

export function SlaStatusBadge({ incidentId, compact = false }: SlaStatusBadgeProps) {
  const { data: slaStatus } = useSlaStatus(incidentId);

  if (!slaStatus || (slaStatus.response_target_minutes === null && slaStatus.resolve_target_minutes === null)) {
    return null; // No SLA configured for this priority
  }

  const anyBreached = slaStatus.response_breached || slaStatus.resolve_breached;
  const responseWarning =
    !slaStatus.response_breached &&
    slaStatus.response_target_minutes !== null &&
    slaStatus.response_elapsed_minutes !== null &&
    (slaStatus.response_elapsed_minutes / slaStatus.response_target_minutes) >= 0.75;
  const resolveWarning =
    !slaStatus.resolve_breached &&
    slaStatus.resolve_target_minutes !== null &&
    slaStatus.resolve_elapsed_minutes !== null &&
    (slaStatus.resolve_elapsed_minutes / slaStatus.resolve_target_minutes) >= 0.75;
  const anyWarning = responseWarning || resolveWarning;

  if (compact) {
    return (
      <Badge
        variant="outline"
        className={
          anyBreached
            ? "bg-red-500/10 text-red-500 border-red-500/20"
            : anyWarning
              ? "bg-yellow-500/10 text-yellow-600 border-yellow-500/20"
              : "bg-green-500/10 text-green-600 border-green-500/20"
        }
      >
        {anyBreached ? "SLA Breached" : anyWarning ? "SLA At Risk" : "SLA OK"}
      </Badge>
    );
  }

  return (
    <div className="space-y-1 rounded-md border p-3">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
          SLA Status ({slaStatus.priority})
        </span>
        <Badge
          variant="outline"
          className={
            anyBreached
              ? "bg-red-500/10 text-red-500 border-red-500/20"
              : anyWarning
                ? "bg-yellow-500/10 text-yellow-600 border-yellow-500/20"
                : "bg-green-500/10 text-green-600 border-green-500/20"
          }
        >
          {anyBreached ? "Breached" : anyWarning ? "At Risk" : "On Track"}
        </Badge>
      </div>
      <SlaIndicator
        label="Response"
        elapsed={slaStatus.response_elapsed_minutes}
        target={slaStatus.response_target_minutes}
        breached={slaStatus.response_breached}
      />
      <SlaIndicator
        label="Resolve"
        elapsed={slaStatus.resolve_elapsed_minutes}
        target={slaStatus.resolve_target_minutes}
        breached={slaStatus.resolve_breached}
      />
    </div>
  );
}
