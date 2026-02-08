import { useState, useEffect, useRef, useCallback } from "react";
import { FileText, Sparkles, Loader2, Save, Plus } from "lucide-react";
import {
  usePostmortemByIncident,
  useCreatePostmortem,
  useUpdatePostmortem,
  usePostmortemTemplates,
} from "@/hooks/use-postmortems";
import { useAiPostmortemDraft, useAiStatus } from "@/hooks/use-ai";
import { useContributingFactors } from "@/hooks/use-postmortems";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { MarkdownEditor } from "@/components/ui/markdown-editor";
import { toast } from "@/components/ui/use-toast";

interface PostmortemEditorProps {
  incidentId: string;
  title: string;
  severity: string;
  service: string;
  rootCause: string;
  resolution: string;
  lessons: string;
}

const PM_STATUS_COLORS: Record<string, string> = {
  draft: "bg-yellow-500/10 text-yellow-600 border-yellow-500/20",
  review: "bg-blue-500/10 text-blue-600 border-blue-500/20",
  final: "bg-green-500/10 text-green-600 border-green-500/20",
};

export function PostmortemEditor({
  incidentId,
  title,
  severity,
  service,
  rootCause,
  resolution,
  lessons,
}: PostmortemEditorProps) {
  const { data: existingPm } = usePostmortemByIncident(incidentId);
  const { data: templates } = usePostmortemTemplates();
  const { data: factors } = useContributingFactors(incidentId);
  const { data: aiStatus } = useAiStatus();
  const createPm = useCreatePostmortem();
  const updatePm = useUpdatePostmortem();
  const aiDraft = useAiPostmortemDraft();

  const [content, setContent] = useState("");
  const [pmStatus, setPmStatus] = useState<"draft" | "review" | "final">(
    "draft"
  );
  const contentRef = useRef(content);
  contentRef.current = content;

  useEffect(() => {
    if (existingPm) {
      // Parse content if it's JSON with a "markdown" field, otherwise use raw
      try {
        const parsed = JSON.parse(existingPm.content);
        setContent(parsed.markdown ?? existingPm.content);
      } catch {
        setContent(existingPm.content === "{}" ? "" : existingPm.content);
      }
      setPmStatus(existingPm.status);
    }
  }, [existingPm]);

  const handleCreate = async (templateId?: string) => {
    try {
      await createPm.mutateAsync({
        incident_id: incidentId,
        template_id: templateId,
      });
      toast({ title: "Post-mortem created" });
    } catch (err) {
      toast({
        title: "Failed to create post-mortem",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleSave = useCallback(async () => {
    if (!existingPm) return;
    try {
      await updatePm.mutateAsync({
        id: existingPm.id,
        req: {
          content: JSON.stringify({ markdown: contentRef.current }),
          status: pmStatus,
        },
      });
      toast({ title: "Post-mortem saved" });
    } catch (err) {
      toast({
        title: "Failed to save",
        description: String(err),
        variant: "destructive",
      });
    }
  }, [existingPm, pmStatus, updatePm]);

  const handleAiDraft = async () => {
    try {
      const factorDescriptions =
        factors?.map((f) => `[${f.category}] ${f.description}`) ?? [];
      const result = await aiDraft.mutateAsync({
        title,
        severity,
        service,
        rootCause,
        resolution,
        lessons,
        contributingFactors: factorDescriptions,
      });
      setContent(result);
      toast({ title: "AI draft generated" });
    } catch (err) {
      toast({
        title: "AI draft failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  // No PM yet — show create button
  if (!existingPm) {
    return (
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-base">
            <FileText className="h-4 w-4" />
            Post-Mortem
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="text-sm text-muted-foreground">
            No post-mortem has been created for this incident.
          </p>
          <div className="flex flex-wrap gap-2">
            {templates?.map((t) => (
              <Button
                key={t.id}
                size="sm"
                variant="outline"
                onClick={() => handleCreate(t.id)}
              >
                <FileText className="mr-1 h-3 w-3" />
                {t.name}
              </Button>
            ))}
            <Button size="sm" onClick={() => handleCreate()}>
              <Plus className="mr-1 h-3 w-3" />
              Blank Post-Mortem
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-base">
            <FileText className="h-4 w-4" />
            Post-Mortem
          </CardTitle>
          <div className="flex items-center gap-2">
            <Badge
              variant="outline"
              className={PM_STATUS_COLORS[pmStatus] ?? ""}
            >
              {pmStatus}
            </Badge>
            <Select
              value={pmStatus}
              onChange={(e) =>
                setPmStatus(e.target.value as "draft" | "review" | "final")
              }
              className="h-7 w-24 text-xs"
            >
              <option value="draft">Draft</option>
              <option value="review">Review</option>
              <option value="final">Final</option>
            </Select>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex gap-2">
          {aiStatus?.available && (
            <Button
              size="sm"
              variant="outline"
              onClick={handleAiDraft}
              disabled={aiDraft.isPending}
            >
              {aiDraft.isPending ? (
                <Loader2 className="mr-1 h-3 w-3 animate-spin" />
              ) : (
                <Sparkles className="mr-1 h-3 w-3" />
              )}
              AI Draft
            </Button>
          )}
          <Button
            size="sm"
            onClick={handleSave}
            disabled={updatePm.isPending}
          >
            <Save className="mr-1 h-3 w-3" />
            Save
          </Button>
        </div>

        <MarkdownEditor value={content} onChange={setContent} />
      </CardContent>
    </Card>
  );
}
