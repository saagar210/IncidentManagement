import { useState } from "react";
import { Copy, Plus, Trash2, Sparkles, Loader2 } from "lucide-react";
import {
  useStakeholderUpdates,
  useCreateStakeholderUpdate,
  useDeleteStakeholderUpdate,
} from "@/hooks/use-stakeholder-updates";
import { useAiStakeholderUpdate, useAiStatus } from "@/hooks/use-ai";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { toast } from "@/components/ui/use-toast";

interface StakeholderUpdatePanelProps {
  incidentId: string;
  title: string;
  severity: string;
  status: string;
  service: string;
  impact: string;
  notes: string;
}

const UPDATE_TYPE_COLORS: Record<string, string> = {
  initial: "bg-blue-500/10 text-blue-600 border-blue-500/20",
  status: "bg-yellow-500/10 text-yellow-600 border-yellow-500/20",
  final: "bg-green-500/10 text-green-600 border-green-500/20",
  custom: "bg-purple-500/10 text-purple-600 border-purple-500/20",
};

const TEMPLATES: Record<string, (props: StakeholderUpdatePanelProps) => string> = {
  initial: (p) =>
    `**Initial Notification — ${p.severity} Incident**\n\nWe are aware of an issue affecting **${p.service}**.\n\n**Status**: ${p.status}\n**Impact**: ${p.impact}\n**Severity**: ${p.severity}\n\nWe are actively investigating and will provide updates as more information becomes available.`,
  status: (p) =>
    `**Status Update — ${p.title}**\n\nService: ${p.service}\nCurrent Status: ${p.status}\nSeverity: ${p.severity}\n\n${p.notes ? `**Latest Notes**: ${p.notes}` : "Investigation is ongoing. Next update in 30 minutes."}`,
  final: (p) =>
    `**Resolution Notice — ${p.title}**\n\nThe incident affecting **${p.service}** has been resolved.\n\n**Final Status**: ${p.status}\n**Severity**: ${p.severity}\n\nA post-mortem review will be conducted. Thank you for your patience.`,
};

export function StakeholderUpdatePanel(props: StakeholderUpdatePanelProps) {
  const { incidentId } = props;
  const { data: updates } = useStakeholderUpdates(incidentId);
  const createMutation = useCreateStakeholderUpdate();
  const deleteMutation = useDeleteStakeholderUpdate();
  const { data: aiStatus } = useAiStatus();
  const aiStakeholder = useAiStakeholderUpdate();

  const [newContent, setNewContent] = useState("");
  const [updateType, setUpdateType] = useState("status");
  const [showCompose, setShowCompose] = useState(false);

  const handleTemplate = (type: string) => {
    const tmpl = TEMPLATES[type];
    if (tmpl) {
      setNewContent(tmpl(props));
      setUpdateType(type);
      setShowCompose(true);
    }
  };

  const handleAiGenerate = async () => {
    try {
      const result = await aiStakeholder.mutateAsync({
        title: props.title,
        severity: props.severity,
        status: props.status,
        service: props.service,
        impact: props.impact,
        notes: props.notes,
      });
      setNewContent(result);
      setUpdateType("status");
      setShowCompose(true);
    } catch (err) {
      toast({
        title: "AI generation failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleSave = async () => {
    if (!newContent.trim()) return;
    try {
      await createMutation.mutateAsync({
        incident_id: incidentId,
        content: newContent.trim(),
        update_type: updateType,
        generated_by: "manual",
      });
      setNewContent("");
      setShowCompose(false);
      toast({ title: "Stakeholder update saved" });
    } catch (err) {
      toast({
        title: "Failed to save update",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const copyToClipboard = async (text: string) => {
    await navigator.clipboard.writeText(text);
    toast({ title: "Copied to clipboard" });
  };

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-base">Stakeholder Updates</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {/* Quick action buttons */}
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => handleTemplate("initial")}
          >
            Initial Notice
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => handleTemplate("status")}
          >
            Status Update
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => handleTemplate("final")}
          >
            Resolution Notice
          </Button>
          {aiStatus?.available && (
            <Button
              size="sm"
              variant="outline"
              onClick={handleAiGenerate}
              disabled={aiStakeholder.isPending}
            >
              {aiStakeholder.isPending ? (
                <Loader2 className="mr-1 h-3 w-3 animate-spin" />
              ) : (
                <Sparkles className="mr-1 h-3 w-3" />
              )}
              AI Generate
            </Button>
          )}
          <Button
            size="sm"
            variant="outline"
            onClick={() => setShowCompose(true)}
          >
            <Plus className="mr-1 h-3 w-3" />
            Custom
          </Button>
        </div>

        {/* Compose area */}
        {showCompose && (
          <div className="space-y-2 rounded-md border p-3">
            <div className="flex items-center gap-2">
              <Select
                value={updateType}
                onChange={(e) => setUpdateType(e.target.value)}
                className="h-7 w-28 text-xs"
              >
                <option value="initial">Initial</option>
                <option value="status">Status</option>
                <option value="final">Final</option>
                <option value="custom">Custom</option>
              </Select>
            </div>
            <textarea
              className="w-full rounded-md border bg-background px-3 py-2 text-sm"
              rows={6}
              value={newContent}
              onChange={(e) => setNewContent(e.target.value)}
              placeholder="Write stakeholder update..."
            />
            <div className="flex justify-end gap-2">
              <Button
                size="sm"
                variant="ghost"
                onClick={() => {
                  setShowCompose(false);
                  setNewContent("");
                }}
              >
                Cancel
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => copyToClipboard(newContent)}
                disabled={!newContent.trim()}
              >
                <Copy className="mr-1 h-3 w-3" />
                Copy
              </Button>
              <Button
                size="sm"
                onClick={handleSave}
                disabled={!newContent.trim() || createMutation.isPending}
              >
                Save
              </Button>
            </div>
          </div>
        )}

        {/* History */}
        {updates && updates.length > 0 && (
          <div className="space-y-2">
            <p className="text-xs font-medium text-muted-foreground">
              History ({updates.length})
            </p>
            {updates.map((u) => (
              <div
                key={u.id}
                className="rounded-md border bg-muted/30 p-3"
              >
                <div className="mb-1 flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Badge
                      variant="outline"
                      className={UPDATE_TYPE_COLORS[u.update_type] ?? ""}
                    >
                      {u.update_type}
                    </Badge>
                    <span className="text-xs text-muted-foreground">
                      {u.generated_by}
                    </span>
                  </div>
                  <div className="flex items-center gap-1">
                    <span className="text-xs text-muted-foreground">
                      {new Date(u.created_at).toLocaleString()}
                    </span>
                    <Button
                      size="sm"
                      variant="ghost"
                      className="h-6 w-6 p-0"
                      onClick={() => copyToClipboard(u.content)}
                    >
                      <Copy className="h-3 w-3" />
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      className="h-6 w-6 p-0 text-muted-foreground hover:text-destructive"
                      onClick={() => deleteMutation.mutate(u.id)}
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
                <p className="whitespace-pre-wrap text-sm">{u.content}</p>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
