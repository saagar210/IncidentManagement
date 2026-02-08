import { useState, useEffect } from "react";
import { Clock, AlertTriangle, CheckCircle } from "lucide-react";
import { cn } from "@/lib/utils";

interface SlaCountdownTimerProps {
  targetMinutes: number;
  elapsedMinutes: number;
  breached: boolean;
  label: string;
}

function formatTimeRemaining(minutes: number): string {
  if (minutes <= 0) return "0m";
  if (minutes < 60) return `${Math.floor(minutes)}m`;
  const hours = Math.floor(minutes / 60);
  const mins = Math.floor(minutes % 60);
  if (hours < 24) return `${hours}h ${mins}m`;
  const days = Math.floor(hours / 24);
  const remainingHours = hours % 24;
  return `${days}d ${remainingHours}h`;
}

export function SlaCountdownTimer({
  targetMinutes,
  elapsedMinutes,
  breached,
  label,
}: SlaCountdownTimerProps) {
  const [now, setNow] = useState(Date.now());

  // Tick every minute for live countdown
  useEffect(() => {
    if (breached) return;
    const interval = setInterval(() => setNow(Date.now()), 60000);
    return () => clearInterval(interval);
  }, [breached]);

  // Suppress unused variable warning — `now` triggers re-render
  void now;

  const remaining = targetMinutes - elapsedMinutes;
  const percentage = Math.min((elapsedMinutes / targetMinutes) * 100, 100);
  const isWarning = !breached && percentage >= 75;

  return (
    <div className="flex items-center gap-2">
      {breached ? (
        <AlertTriangle className="h-4 w-4 shrink-0 text-red-500" />
      ) : isWarning ? (
        <Clock className="h-4 w-4 shrink-0 text-yellow-500" />
      ) : (
        <CheckCircle className="h-4 w-4 shrink-0 text-green-500" />
      )}
      <div className="flex-1 space-y-1">
        <div className="flex items-center justify-between text-xs">
          <span className="font-medium">{label}</span>
          <span
            className={cn(
              breached
                ? "text-red-500"
                : isWarning
                  ? "text-yellow-600"
                  : "text-muted-foreground"
            )}
          >
            {breached
              ? `Breached by ${formatTimeRemaining(Math.abs(remaining))}`
              : `${formatTimeRemaining(remaining)} left`}
          </span>
        </div>
        <div className="h-1.5 rounded-full bg-muted">
          <div
            className={cn(
              "h-full rounded-full transition-all",
              breached
                ? "bg-red-500"
                : isWarning
                  ? "bg-yellow-500"
                  : "bg-green-500"
            )}
            style={{ width: `${Math.min(percentage, 100)}%` }}
          />
        </div>
      </div>
    </div>
  );
}
