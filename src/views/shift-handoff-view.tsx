import { useState } from "react";
import { ArrowRightLeft, Plus, Trash2, Clock } from "lucide-react";
import {
  useShiftHandoffs,
  useCreateShiftHandoff,
  useDeleteShiftHandoff,
} from "@/hooks/use-shift-handoffs";
import { useQuery } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { toast } from "@/components/ui/use-toast";
import { SEVERITY_COLORS } from "@/lib/constants";
import type { SeverityLevel } from "@/lib/constants";
import type { Incident } from "@/types/incident";

export function ShiftHandoffView() {
  const { data: handoffs } = useShiftHandoffs(20);
  const createHandoff = useCreateShiftHandoff();
  const deleteHandoff = useDeleteShiftHandoff();
  const [showCreate, setShowCreate] = useState(false);
  const [createdBy, setCreatedBy] = useState("");
  const [customNotes, setCustomNotes] = useState("");

  // Fetch active incidents for auto-generated handoff content
  const { data: activeIncidents } = useQuery({
    queryKey: ["active-incidents-for-handoff"],
    queryFn: () =>
      tauriInvoke<Incident[]>("list_incidents", {
        filters: { status: "Active" },
        quarterId: null,
      }),
  });

  const generateContent = () => {
    const sections: string[] = [];

    if (activeIncidents && activeIncidents.length > 0) {
      sections.push("## Active Incidents\n");
      for (const inc of activeIncidents) {
        sections.push(
          `- **${inc.title}** (${inc.severity}, ${inc.service_name}) — ${inc.status}`
        );
      }
    } else {
      sections.push("## Active Incidents\n\nNo active incidents.");
    }

    if (customNotes.trim()) {
      sections.push(`\n## Notes\n\n${customNotes.trim()}`);
    }

    return sections.join("\n");
  };

  const handleCreate = async () => {
    try {
      const content = generateContent();
      await createHandoff.mutateAsync({
        content,
        created_by: createdBy.trim() || undefined,
      });
      setShowCreate(false);
      setCreatedBy("");
      setCustomNotes("");
      toast({ title: "Shift handoff created" });
    } catch (err) {
      toast({
        title: "Failed to create handoff",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteHandoff.mutateAsync(id);
    } catch (err) {
      toast({
        title: "Failed to delete",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <ArrowRightLeft className="h-6 w-6 text-blue-500" />
          <h1 className="text-2xl font-semibold">Shift Handoffs</h1>
        </div>
        <Button onClick={() => setShowCreate(true)} disabled={showCreate}>
          <Plus className="h-4 w-4" />
          New Handoff
        </Button>
      </div>

      <p className="text-sm text-muted-foreground">
        Create a shift handoff report to summarize active incidents and pass
        context to the next responder.
      </p>

      {/* Create Form */}
      {showCreate && (
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">New Shift Handoff</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <Label>Your Name (optional)</Label>
              <Input
                value={createdBy}
                onChange={(e) => setCreatedBy(e.target.value)}
                placeholder="e.g., Jane Smith"
              />
            </div>

            {/* Active incidents preview */}
            {activeIncidents && activeIncidents.length > 0 && (
              <div>
                <Label>Active Incidents (auto-included)</Label>
                <div className="mt-1 space-y-1">
                  {activeIncidents.map((inc) => (
                    <div
                      key={inc.id}
                      className="flex items-center gap-2 rounded border p-2 text-sm"
                    >
                      <Badge
                        variant="outline"
                        className={
                          SEVERITY_COLORS[inc.severity as SeverityLevel] ?? ""
                        }
                      >
                        {inc.severity}
                      </Badge>
                      <span className="flex-1 truncate">{inc.title}</span>
                      <span className="text-xs text-muted-foreground">
                        {inc.service_name}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div>
              <Label>Additional Notes</Label>
              <textarea
                className="w-full rounded-md border bg-background px-3 py-2 text-sm"
                rows={4}
                value={customNotes}
                onChange={(e) => setCustomNotes(e.target.value)}
                placeholder="Any additional context for the next shift..."
              />
            </div>

            <div className="flex justify-end gap-2">
              <Button
                variant="ghost"
                onClick={() => {
                  setShowCreate(false);
                  setCreatedBy("");
                  setCustomNotes("");
                }}
              >
                Cancel
              </Button>
              <Button
                onClick={handleCreate}
                disabled={createHandoff.isPending}
              >
                Create Handoff
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Handoff History */}
      {handoffs && handoffs.length > 0 ? (
        <div className="space-y-3">
          {handoffs.map((h) => (
            <Card key={h.id}>
              <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Clock className="h-4 w-4 text-muted-foreground" />
                    <span className="text-sm font-medium">
                      {new Date(h.created_at).toLocaleString()}
                    </span>
                    {h.created_by && (
                      <Badge variant="outline">{h.created_by}</Badge>
                    )}
                  </div>
                  <Button
                    size="sm"
                    variant="ghost"
                    className="h-7 w-7 p-0 text-muted-foreground hover:text-destructive"
                    onClick={() => handleDelete(h.id)}
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </Button>
                </div>
              </CardHeader>
              <CardContent>
                <p className="whitespace-pre-wrap text-sm">{h.content}</p>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        !showCreate && (
          <div className="rounded-lg border border-dashed p-8 text-center">
            <ArrowRightLeft className="mx-auto mb-3 h-10 w-10 text-muted-foreground/50" />
            <p className="text-sm text-muted-foreground">
              No shift handoffs yet. Create one to document the current state for
              the next responder.
            </p>
          </div>
        )
      )}
    </div>
  );
}
