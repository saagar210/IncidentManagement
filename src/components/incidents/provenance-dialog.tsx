import { useMemo } from "react";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { useFieldProvenance } from "@/hooks/use-provenance";

interface ProvenanceDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  entityType: string;
  entityId: string | null;
}

export function ProvenanceDialog({ open, onOpenChange, entityType, entityId }: ProvenanceDialogProps) {
  const { data, isLoading } = useFieldProvenance(entityType, entityId);

  const rows = useMemo(() => (data ?? []).slice(0, 200), [data]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Provenance</DialogTitle>
          <DialogDescription>
            Field-level provenance for this entity (imported, manual, computed, or AI-enriched).
          </DialogDescription>
        </DialogHeader>

        {isLoading ? (
          <div className="text-sm text-muted-foreground">Loading provenance…</div>
        ) : rows.length === 0 ? (
          <div className="text-sm text-muted-foreground">No provenance recorded yet.</div>
        ) : (
          <div className="max-h-[420px] overflow-y-auto rounded border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Field</TableHead>
                  <TableHead>Source</TableHead>
                  <TableHead>Ref</TableHead>
                  <TableHead>When</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.map((p) => (
                  <TableRow key={p.id}>
                    <TableCell className="font-medium">{p.field_name}</TableCell>
                    <TableCell>
                      <Badge variant="outline">{p.source_type}</Badge>
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {p.source_ref || "--"}
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {p.recorded_at}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}

