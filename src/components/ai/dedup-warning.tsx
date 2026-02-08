import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { AlertTriangle } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import { useAiStatus } from "@/hooks/use-ai";
import { Badge } from "@/components/ui/badge";
import type { SimilarIncident } from "@/types/ai";

interface DedupWarningProps {
  title: string;
  serviceId: string;
}

export function DedupWarning({ title, serviceId }: DedupWarningProps) {
  const navigate = useNavigate();
  const { data: aiStatus } = useAiStatus();

  const { data: duplicates, refetch } = useQuery({
    queryKey: ["dedup-check", title, serviceId],
    queryFn: () =>
      tauriInvoke<SimilarIncident[]>("check_duplicate_incidents", {
        title,
        serviceId,
      }),
    enabled: false, // manual trigger only
  });

  // Trigger check when title has 5+ chars and a service is selected
  useEffect(() => {
    if (title.trim().length >= 5 && serviceId) {
      const timer = setTimeout(() => {
        refetch();
      }, 500);
      return () => clearTimeout(timer);
    }
  }, [title, serviceId, refetch]);

  if (!aiStatus?.available || !duplicates || duplicates.length === 0) {
    return null;
  }

  return (
    <div className="rounded-md border border-yellow-500/50 bg-yellow-50 p-3 dark:bg-yellow-950/20">
      <div className="mb-2 flex items-center gap-2 text-sm font-medium text-yellow-800 dark:text-yellow-200">
        <AlertTriangle className="h-4 w-4" />
        Possible duplicate incidents found
      </div>
      <ul className="space-y-1.5">
        {duplicates.map((dup) => (
          <li key={dup.id} className="flex items-center gap-2 text-sm">
            <button
              type="button"
              className="truncate text-left text-blue-600 underline-offset-2 hover:underline dark:text-blue-400"
              onClick={() => navigate(`/incidents/${dup.id}`)}
            >
              {dup.title}
            </button>
            <Badge variant="outline" className="shrink-0 text-[10px]">
              {dup.severity}
            </Badge>
            <span className="shrink-0 text-xs text-muted-foreground">
              {dup.status}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
