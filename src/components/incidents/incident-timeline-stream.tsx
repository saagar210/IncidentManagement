import { format } from "date-fns";
import { useMemo, useState, useCallback } from "react";
import {
  AlertCircle,
  CheckCircle,
  Edit,
  UserPlus,
  ListChecks,
  MessageSquare,
  Plus,
  Upload,
  Trash2,
} from "lucide-react";
import { useEntityAuditLog } from "@/hooks/use-audit";
import { useTimelineEvents, useCreateTimelineEvent, useDeleteTimelineEvent, useImportTimelineFromPaste } from "@/hooks/use-timeline-events";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { toast } from "@/components/ui/use-toast";
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
  const { data: entries, isLoading: auditLoading } = useEntityAuditLog("incident", incidentId);
  const { data: timeline, isLoading: timelineLoading } = useTimelineEvents(incidentId);
  const createEvent = useCreateTimelineEvent();
  const deleteEvent = useDeleteTimelineEvent();
  const importPaste = useImportTimelineFromPaste();

  const [addOpen, setAddOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [occurredAt, setOccurredAt] = useState("");
  const [actor, setActor] = useState("");
  const [message, setMessage] = useState("");
  const [pasteText, setPasteText] = useState("");

  const isLoading = auditLoading || timelineLoading;
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

  type UnifiedEvent =
    | {
        kind: "audit";
        id: string;
        ts: string;
        badge: string;
        summary: string;
        details?: string;
        action: string;
      }
    | {
        kind: "timeline";
        id: string;
        ts: string;
        badge: string;
        summary: string;
        details?: string;
      };

  const unified = useMemo<UnifiedEvent[]>(() => {
    const out: UnifiedEvent[] = [];
    for (const e of entries ?? []) {
      out.push({
        kind: "audit",
        id: e.id,
        ts: e.created_at,
        badge: e.action.replace(/_/g, " "),
        summary: e.summary,
        details: e.details || undefined,
        action: e.action,
      });
    }
    for (const t of timeline ?? []) {
      const who = t.actor ? ` (${t.actor})` : "";
      out.push({
        kind: "timeline",
        id: t.id,
        ts: t.occurred_at,
        badge: t.source || "timeline",
        summary: `${t.message}${who}`,
      });
    }
    out.sort((a, b) => new Date(b.ts).getTime() - new Date(a.ts).getTime());
    return out;
  }, [entries, timeline]);

  const handleAdd = useCallback(async () => {
    try {
      await createEvent.mutateAsync({
        incident_id: incidentId,
        occurred_at: occurredAt,
        source: "manual",
        message,
        actor: actor.trim() ? actor.trim() : undefined,
      });
      setAddOpen(false);
      setOccurredAt("");
      setActor("");
      setMessage("");
      toast({ title: "Timeline event added" });
    } catch (err) {
      toast({ title: "Failed to add event", description: String(err), variant: "destructive" });
    }
  }, [createEvent, incidentId, occurredAt, actor, message]);

  const handleImportPaste = useCallback(async () => {
    try {
      const res = await importPaste.mutateAsync({
        incident_id: incidentId,
        paste_text: pasteText,
        source: "paste",
      });
      setImportOpen(false);
      setPasteText("");
      toast({ title: "Timeline imported", description: `Created ${res.created}, skipped ${res.skipped}` });
    } catch (err) {
      toast({ title: "Import failed", description: String(err), variant: "destructive" });
    }
  }, [importPaste, incidentId, pasteText]);

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">
          Timeline
          {unified.length > 0 && (
            <span className="ml-2 text-sm font-normal text-muted-foreground">
              {unified.length} events
            </span>
          )}
        </CardTitle>
        <div className="mt-2 flex gap-2">
          <Button size="sm" variant="outline" onClick={() => setAddOpen(true)}>
            <Plus className="h-4 w-4 mr-2" />
            Add
          </Button>
          <Button size="sm" variant="outline" onClick={() => setImportOpen(true)}>
            <Upload className="h-4 w-4 mr-2" />
            Import Paste
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {unified.length === 0 ? (
          <p className="flex h-24 items-center justify-center text-sm text-muted-foreground">
            No timeline events yet
          </p>
        ) : (
          <div className="relative space-y-0">
            {unified.map((entry, index) => (
              <div key={entry.id} className="flex gap-3 pb-4">
                {/* Timeline connector */}
                <div className="relative flex flex-col items-center">
                  <div
                    className={cn(
                      "flex h-8 w-8 shrink-0 items-center justify-center rounded-full border-2 bg-background",
                      entry.kind === "audit" ? getEventColor(entry.action) : "border-muted-foreground"
                    )}
                  >
                    {entry.kind === "audit" ? getEventIcon(entry.action) : <MessageSquare className="h-4 w-4 text-muted-foreground" />}
                  </div>
                  {index < unified.length - 1 && (
                    <div className="w-px flex-1 bg-border" />
                  )}
                </div>

                {/* Event content */}
                <div className="flex-1 pt-1">
                  <div className="flex items-center gap-2">
                    <Badge variant="outline" className="text-xs">
                      {entry.badge}
                    </Badge>
                    <span className="text-xs text-muted-foreground">
                      {formatTimestamp(entry.ts)}
                    </span>
                    {entry.kind === "timeline" ? (
                      <Button
                        size="icon"
                        variant="ghost"
                        className="ml-auto h-7 w-7"
                        onClick={() => deleteEvent.mutateAsync({ id: entry.id, incidentId })}
                        disabled={deleteEvent.isPending}
                      >
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    ) : null}
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

      <Dialog open={addOpen} onOpenChange={setAddOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add Timeline Event</DialogTitle>
            <DialogDescription>
              Timestamp formats: RFC3339 (2026-01-01T10:07:00Z) or "YYYY-MM-DD HH:MM".
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-1">
              <label className="text-sm font-medium">Occurred At</label>
              <Input value={occurredAt} onChange={(e) => setOccurredAt(e.target.value)} placeholder="2026-01-01T10:07:00Z" />
            </div>
            <div className="space-y-1">
              <label className="text-sm font-medium">Actor (optional)</label>
              <Input value={actor} onChange={(e) => setActor(e.target.value)} placeholder="oncall@example.com" />
            </div>
            <div className="space-y-1">
              <label className="text-sm font-medium">Message</label>
              <Textarea value={message} onChange={(e) => setMessage(e.target.value)} placeholder="What happened?" />
            </div>
            <div className="flex justify-end gap-2">
              <Button variant="outline" onClick={() => setAddOpen(false)}>Cancel</Button>
              <Button onClick={handleAdd} disabled={!occurredAt.trim() || !message.trim() || createEvent.isPending}>Add</Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog open={importOpen} onOpenChange={setImportOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Import Timeline From Paste</DialogTitle>
            <DialogDescription>
              One event per line: "YYYY-MM-DD HH:MM &lt;text&gt;" or "YYYY-MM-DDTHH:MM:SSZ &lt;text&gt;".
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-3">
            <Textarea value={pasteText} onChange={(e) => setPasteText(e.target.value)} placeholder="2026-01-01 10:06 Detected elevated error rate\n2026-01-01T10:07:00Z Paged oncall" />
            <div className="flex justify-end gap-2">
              <Button variant="outline" onClick={() => setImportOpen(false)}>Cancel</Button>
              <Button onClick={handleImportPaste} disabled={!pasteText.trim() || importPaste.isPending}>Import</Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </Card>
  );
}
