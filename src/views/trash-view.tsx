import { format } from "date-fns";
import { RotateCcw, Trash2 } from "lucide-react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/skeleton";
import { toast } from "@/components/ui/use-toast";
import { SEVERITY_COLORS } from "@/lib/constants";
import type { Incident } from "@/types/incident";
import type { SeverityLevel } from "@/lib/constants";

function formatDate(dateStr: string): string {
  try {
    return format(new Date(dateStr), "MMM d, yyyy HH:mm");
  } catch {
    return dateStr;
  }
}

export function TrashView() {
  const qc = useQueryClient();

  const { data: deleted, isLoading } = useQuery({
    queryKey: ["deleted-incidents"],
    queryFn: () => tauriInvoke<Incident[]>("list_deleted_incidents"),
  });

  const restoreMutation = useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<Incident>("restore_incident", { id }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["deleted-incidents"] });
      qc.invalidateQueries({ queryKey: ["incidents"] });
      qc.invalidateQueries({ queryKey: ["deleted-count"] });
      toast({ title: "Incident restored" });
    },
  });

  const permanentDeleteMutation = useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("permanent_delete_incident", { id }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["deleted-incidents"] });
      qc.invalidateQueries({ queryKey: ["deleted-count"] });
      toast({ title: "Incident permanently deleted" });
    },
  });

  const handlePermanentDelete = (incident: Incident) => {
    const confirmed = window.confirm(
      `Permanently delete "${incident.title}"? This cannot be undone.`
    );
    if (!confirmed) return;
    permanentDeleteMutation.mutate(incident.id);
  };

  return (
    <div className="space-y-4 p-6">
      <div>
        <h1 className="text-2xl font-semibold">Trash</h1>
        <p className="text-sm text-muted-foreground">
          Deleted incidents are kept for 30 days before permanent removal.
        </p>
      </div>

      {isLoading ? (
        <div className="space-y-2">
          {Array.from({ length: 5 }).map((_, i) => (
            <Skeleton key={i} className="h-10 w-full" />
          ))}
        </div>
      ) : !deleted || deleted.length === 0 ? (
        <div className="flex h-64 items-center justify-center text-muted-foreground">
          <p>Trash is empty.</p>
        </div>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Title</TableHead>
              <TableHead>Service</TableHead>
              <TableHead>Severity</TableHead>
              <TableHead>Started At</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {deleted.map((incident) => (
              <TableRow key={incident.id}>
                <TableCell className="font-medium max-w-[300px] truncate">
                  {incident.title}
                </TableCell>
                <TableCell>{incident.service_name}</TableCell>
                <TableCell>
                  <Badge
                    variant="outline"
                    className={
                      SEVERITY_COLORS[incident.severity as SeverityLevel] ?? ""
                    }
                  >
                    {incident.severity}
                  </Badge>
                </TableCell>
                <TableCell className="whitespace-nowrap text-sm">
                  {formatDate(incident.started_at)}
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-1">
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => restoreMutation.mutate(incident.id)}
                      disabled={restoreMutation.isPending}
                    >
                      <RotateCcw className="h-4 w-4" />
                      Restore
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => handlePermanentDelete(incident)}
                      disabled={permanentDeleteMutation.isPending}
                    >
                      <Trash2 className="h-4 w-4 text-destructive" />
                      Delete
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </div>
  );
}
