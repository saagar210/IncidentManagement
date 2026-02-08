import { useState } from "react";
import { Plus, Trash2, CheckSquare, Square, ListChecks } from "lucide-react";
import {
  useIncidentChecklists,
  useCreateIncidentChecklist,
  useDeleteIncidentChecklist,
  useToggleChecklistItem,
  useChecklistTemplates,
} from "@/hooks/use-checklists";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { toast } from "@/components/ui/use-toast";

interface ChecklistPanelProps {
  incidentId: string;
}

export function ChecklistPanel({ incidentId }: ChecklistPanelProps) {
  const { data: checklists, isLoading } = useIncidentChecklists(incidentId);
  const { data: templates } = useChecklistTemplates();
  const createChecklist = useCreateIncidentChecklist();
  const deleteChecklist = useDeleteIncidentChecklist();
  const toggleItem = useToggleChecklistItem();

  const [addMode, setAddMode] = useState<"none" | "template" | "custom">("none");
  const [selectedTemplate, setSelectedTemplate] = useState("");
  const [customName, setCustomName] = useState("");
  const [customItems, setCustomItems] = useState("");

  const handleAddFromTemplate = async () => {
    if (!selectedTemplate) return;
    try {
      await createChecklist.mutateAsync({
        incident_id: incidentId,
        template_id: selectedTemplate,
      });
      setAddMode("none");
      setSelectedTemplate("");
      toast({ title: "Checklist added from template" });
    } catch (err) {
      toast({
        title: "Failed to add checklist",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleAddCustom = async () => {
    const items = customItems
      .split("\n")
      .map((s) => s.trim())
      .filter(Boolean);
    if (items.length === 0) return;
    try {
      await createChecklist.mutateAsync({
        incident_id: incidentId,
        name: customName.trim() || "Checklist",
        items,
      });
      setAddMode("none");
      setCustomName("");
      setCustomItems("");
      toast({ title: "Checklist created" });
    } catch (err) {
      toast({
        title: "Failed to create checklist",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleToggle = async (itemId: string) => {
    try {
      await toggleItem.mutateAsync({ itemId, req: {} });
    } catch (err) {
      toast({
        title: "Failed to toggle item",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleDelete = async (checklistId: string) => {
    try {
      await deleteChecklist.mutateAsync(checklistId);
      toast({ title: "Checklist removed" });
    } catch (err) {
      toast({
        title: "Failed to delete checklist",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  if (isLoading) {
    return <p className="text-sm text-muted-foreground">Loading checklists...</p>;
  }

  const activeTemplates = (templates ?? []).filter((t) => t.is_active);

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="flex items-center gap-1.5 text-sm font-medium">
          <ListChecks className="h-4 w-4" />
          Checklists
        </h3>
        {addMode === "none" && (
          <div className="flex gap-1">
            {activeTemplates.length > 0 && (
              <Button
                size="sm"
                variant="outline"
                onClick={() => setAddMode("template")}
              >
                <Plus className="mr-1 h-3 w-3" />
                From Template
              </Button>
            )}
            <Button
              size="sm"
              variant="outline"
              onClick={() => setAddMode("custom")}
            >
              <Plus className="mr-1 h-3 w-3" />
              Custom
            </Button>
          </div>
        )}
      </div>

      {addMode === "template" && (
        <div className="space-y-2 rounded border p-2">
          <Select
            value={selectedTemplate}
            onChange={(e) => setSelectedTemplate(e.target.value)}
          >
            <option value="">Select template...</option>
            {activeTemplates.map((t) => (
              <option key={t.id} value={t.id}>
                {t.name} ({t.items.length} items)
              </option>
            ))}
          </Select>
          <div className="flex gap-1">
            <Button
              size="sm"
              onClick={handleAddFromTemplate}
              disabled={!selectedTemplate || createChecklist.isPending}
            >
              Add
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setAddMode("none")}
            >
              Cancel
            </Button>
          </div>
        </div>
      )}

      {addMode === "custom" && (
        <div className="space-y-2 rounded border p-2">
          <Input
            value={customName}
            onChange={(e) => setCustomName(e.target.value)}
            placeholder="Checklist name"
          />
          <textarea
            value={customItems}
            onChange={(e) => setCustomItems(e.target.value)}
            placeholder="One item per line..."
            rows={4}
            className="w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm"
          />
          <div className="flex gap-1">
            <Button
              size="sm"
              onClick={handleAddCustom}
              disabled={
                !customItems.trim() || createChecklist.isPending
              }
            >
              Create
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => {
                setAddMode("none");
                setCustomName("");
                setCustomItems("");
              }}
            >
              Cancel
            </Button>
          </div>
        </div>
      )}

      {(checklists ?? []).length === 0 && addMode === "none" && (
        <p className="text-sm text-muted-foreground">
          No checklists attached.
        </p>
      )}

      {(checklists ?? []).map((cl) => {
        const total = cl.items.length;
        const checked = cl.items.filter((i) => i.is_checked).length;
        return (
          <div key={cl.id} className="rounded border">
            <div className="flex items-center justify-between border-b px-3 py-2">
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium">{cl.name}</span>
                <Badge
                  variant={checked === total ? "default" : "secondary"}
                  className="text-[10px]"
                >
                  {checked}/{total}
                </Badge>
              </div>
              <Button
                size="icon"
                variant="ghost"
                className="h-6 w-6"
                onClick={() => handleDelete(cl.id)}
                disabled={deleteChecklist.isPending}
              >
                <Trash2 className="h-3 w-3 text-destructive" />
              </Button>
            </div>
            <div className="divide-y">
              {cl.items.map((item) => (
                <button
                  key={item.id}
                  className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-muted/50 transition-colors"
                  onClick={() => handleToggle(item.id)}
                  disabled={toggleItem.isPending}
                >
                  {item.is_checked ? (
                    <CheckSquare className="h-4 w-4 text-green-500 flex-shrink-0" />
                  ) : (
                    <Square className="h-4 w-4 text-muted-foreground flex-shrink-0" />
                  )}
                  <span
                    className={`text-sm ${
                      item.is_checked
                        ? "line-through text-muted-foreground"
                        : ""
                    }`}
                  >
                    {item.label}
                  </span>
                </button>
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}
