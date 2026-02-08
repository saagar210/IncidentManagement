import { useState, useCallback } from "react";
import { isPast, isWithinInterval, addDays, format } from "date-fns";
import { Plus, Trash2, CheckCircle2, Circle, Clock } from "lucide-react";
import {
  useActionItems,
  useCreateActionItem,
  useUpdateActionItem,
  useDeleteActionItem,
} from "@/hooks/use-incidents";
import { useToast } from "@/components/ui/use-toast";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { ActionItem } from "@/types/incident";
import type { ActionItemStatus } from "@/lib/constants";

// --- Status helpers ---

const STATUS_CYCLE: Record<ActionItemStatus, ActionItemStatus> = {
  Open: "In-Progress",
  "In-Progress": "Done",
  Done: "Open",
};

function getStatusBadgeProps(status: string): {
  variant: "outline" | "secondary" | "default";
  className: string;
  icon: React.ReactNode;
} {
  switch (status) {
    case "In-Progress":
      return {
        variant: "secondary",
        className: "bg-blue-500/10 text-blue-600 border-blue-500/20 cursor-pointer select-none",
        icon: <Clock className="mr-1 h-3 w-3" />,
      };
    case "Done":
      return {
        variant: "default",
        className: "bg-green-500/10 text-green-600 border-green-500/20 cursor-pointer select-none",
        icon: <CheckCircle2 className="mr-1 h-3 w-3" />,
      };
    default:
      return {
        variant: "outline",
        className: "cursor-pointer select-none",
        icon: <Circle className="mr-1 h-3 w-3" />,
      };
  }
}

// --- Due date helpers ---

function getDueDateClasses(dueDate: string | null): string {
  if (!dueDate) return "text-muted-foreground";

  const due = new Date(dueDate);
  const now = new Date();

  if (isPast(due) && due < now) {
    return "text-red-500 font-medium";
  }

  if (isWithinInterval(due, { start: now, end: addDays(now, 3) })) {
    return "text-amber-500 font-medium";
  }

  return "text-muted-foreground";
}

function formatDueDate(dueDate: string | null): string {
  if (!dueDate) return "";
  return format(new Date(dueDate), "MMM d, yyyy h:mm a");
}

// --- Inline form state ---

interface NewItemForm {
  title: string;
  owner: string;
  dueDate: string;
}

const EMPTY_FORM: NewItemForm = { title: "", owner: "", dueDate: "" };

// --- Action Item Row ---

interface ActionItemRowProps {
  item: ActionItem;
  onCycleStatus: (item: ActionItem) => void;
  onUpdateTitle: (id: string, title: string) => void;
  onDelete: (id: string) => void;
  isMutating: boolean;
}

function ActionItemRow({
  item,
  onCycleStatus,
  onUpdateTitle,
  onDelete,
  isMutating,
}: ActionItemRowProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editTitle, setEditTitle] = useState(item.title);

  const statusBadge = getStatusBadgeProps(item.status);
  const isDone = item.status === "Done";

  const handleDoubleClick = useCallback(() => {
    setEditTitle(item.title);
    setIsEditing(true);
  }, [item.title]);

  const handleSaveTitle = useCallback(() => {
    const trimmed = editTitle.trim();
    if (trimmed && trimmed !== item.title) {
      onUpdateTitle(item.id, trimmed);
    }
    setIsEditing(false);
  }, [editTitle, item.title, item.id, onUpdateTitle]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter") {
        handleSaveTitle();
      } else if (e.key === "Escape") {
        setIsEditing(false);
        setEditTitle(item.title);
      }
    },
    [handleSaveTitle, item.title]
  );

  return (
    <div className="flex items-center gap-3 rounded-md border px-3 py-2 transition-colors hover:bg-muted/50">
      {/* Status badge */}
      <Badge
        variant={statusBadge.variant}
        className={statusBadge.className}
        onClick={() => onCycleStatus(item)}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            onCycleStatus(item);
          }
        }}
      >
        {statusBadge.icon}
        {item.status}
      </Badge>

      {/* Title + Owner + Due date */}
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        {isEditing ? (
          <Input
            value={editTitle}
            onChange={(e) => setEditTitle(e.target.value)}
            onBlur={handleSaveTitle}
            onKeyDown={handleKeyDown}
            className="h-7 text-sm"
            autoFocus
          />
        ) : (
          <span
            className={`truncate text-sm ${isDone ? "text-muted-foreground line-through" : ""}`}
            onDoubleClick={handleDoubleClick}
            title="Double-click to edit"
          >
            {item.title}
          </span>
        )}

        <div className="flex items-center gap-3">
          {item.owner && (
            <span className="text-xs text-muted-foreground">{item.owner}</span>
          )}
          {item.due_date && (
            <span className={`text-xs ${getDueDateClasses(item.due_date)}`}>
              {formatDueDate(item.due_date)}
            </span>
          )}
        </div>
      </div>

      {/* Delete button */}
      <Button
        variant="ghost"
        size="icon"
        className="h-7 w-7 shrink-0 text-muted-foreground hover:text-destructive"
        onClick={() => onDelete(item.id)}
        disabled={isMutating}
        aria-label={`Delete action item: ${item.title}`}
      >
        <Trash2 className="h-3.5 w-3.5" />
      </Button>
    </div>
  );
}

// --- Main Panel ---

interface ActionItemsPanelProps {
  incidentId: string;
}

export function ActionItemsPanel({ incidentId }: ActionItemsPanelProps) {
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<NewItemForm>(EMPTY_FORM);

  const { toast } = useToast();
  const { data: items, isLoading, isError, error } = useActionItems(incidentId);
  const createMutation = useCreateActionItem();
  const updateMutation = useUpdateActionItem();
  const deleteMutation = useDeleteActionItem();

  const isMutating =
    createMutation.isPending ||
    updateMutation.isPending ||
    deleteMutation.isPending;

  // --- Handlers ---

  const handleCycleStatus = useCallback(
    (item: ActionItem) => {
      const currentStatus = item.status as ActionItemStatus;
      const nextStatus = STATUS_CYCLE[currentStatus] ?? "Open";
      updateMutation.mutate(
        { id: item.id, item: { status: nextStatus } },
        {
          onSuccess: () => {
            toast({ title: `Status changed to ${nextStatus}` });
          },
          onError: (err) => {
            toast({
              title: "Failed to update status",
              description: String(err),
              variant: "destructive",
            });
          },
        }
      );
    },
    [updateMutation, toast]
  );

  const handleUpdateTitle = useCallback(
    (id: string, title: string) => {
      updateMutation.mutate(
        { id, item: { title } },
        {
          onSuccess: () => {
            toast({ title: "Title updated" });
          },
          onError: (err) => {
            toast({
              title: "Failed to update title",
              description: String(err),
              variant: "destructive",
            });
          },
        }
      );
    },
    [updateMutation, toast]
  );

  const handleDelete = useCallback(
    (id: string) => {
      deleteMutation.mutate(id, {
        onSuccess: () => {
          toast({ title: "Action item deleted" });
        },
        onError: (err) => {
          toast({
            title: "Failed to delete action item",
            description: String(err),
            variant: "destructive",
          });
        },
      });
    },
    [deleteMutation, toast]
  );

  const handleCreate = useCallback(() => {
    const trimmedTitle = form.title.trim();
    if (!trimmedTitle) return;

    createMutation.mutate(
      {
        incident_id: incidentId,
        title: trimmedTitle,
        owner: form.owner.trim() || undefined,
        due_date: form.dueDate || null,
        status: "Open",
      },
      {
        onSuccess: () => {
          toast({ title: "Action item created" });
          setForm(EMPTY_FORM);
          setShowForm(false);
        },
        onError: (err) => {
          toast({
            title: "Failed to create action item",
            description: String(err),
            variant: "destructive",
          });
        },
      }
    );
  }, [form, incidentId, createMutation, toast]);

  const handleCancelForm = useCallback(() => {
    setForm(EMPTY_FORM);
    setShowForm(false);
  }, []);

  // --- Render ---

  const itemCount = items?.length ?? 0;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
        <div className="flex items-center gap-2">
          <CardTitle className="text-base">Action Items</CardTitle>
          <Badge variant="secondary" className="text-xs">
            {itemCount}
          </Badge>
        </div>
      </CardHeader>

      <CardContent className="space-y-3">
        {/* Loading state */}
        {isLoading && (
          <div className="py-6 text-center text-sm text-muted-foreground">
            Loading action items...
          </div>
        )}

        {/* Error state */}
        {isError && (
          <div className="rounded-md border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
            Failed to load action items: {String(error)}
          </div>
        )}

        {/* Empty state */}
        {!isLoading && !isError && itemCount === 0 && !showForm && (
          <div className="py-6 text-center text-sm text-muted-foreground">
            No action items yet. Track follow-up work by adding action items.
          </div>
        )}

        {/* Items list */}
        {!isLoading && items && items.length > 0 && (
          <div className="space-y-2">
            {items.map((item) => (
              <ActionItemRow
                key={item.id}
                item={item}
                onCycleStatus={handleCycleStatus}
                onUpdateTitle={handleUpdateTitle}
                onDelete={handleDelete}
                isMutating={isMutating}
              />
            ))}
          </div>
        )}

        {/* Inline add form */}
        {showForm && (
          <div className="space-y-3 rounded-md border border-dashed p-3">
            <Input
              placeholder="Action item title (required)"
              value={form.title}
              onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))}
              autoFocus
              onKeyDown={(e) => {
                if (e.key === "Enter" && form.title.trim()) {
                  handleCreate();
                } else if (e.key === "Escape") {
                  handleCancelForm();
                }
              }}
            />
            <div className="flex gap-2">
              <Input
                placeholder="Owner"
                value={form.owner}
                onChange={(e) =>
                  setForm((f) => ({ ...f, owner: e.target.value }))
                }
                className="flex-1"
              />
              <Input
                type="datetime-local"
                value={form.dueDate}
                onChange={(e) =>
                  setForm((f) => ({ ...f, dueDate: e.target.value }))
                }
                className="flex-1"
              />
            </div>
            <div className="flex justify-end gap-2">
              <Button
                variant="ghost"
                size="sm"
                onClick={handleCancelForm}
                disabled={createMutation.isPending}
              >
                Cancel
              </Button>
              <Button
                size="sm"
                onClick={handleCreate}
                disabled={!form.title.trim() || createMutation.isPending}
              >
                {createMutation.isPending ? "Saving..." : "Save"}
              </Button>
            </div>
          </div>
        )}

        {/* Add button */}
        {!showForm && (
          <Button
            variant="outline"
            size="sm"
            className="w-full"
            onClick={() => setShowForm(true)}
          >
            <Plus className="mr-1 h-4 w-4" />
            Add Action Item
          </Button>
        )}
      </CardContent>
    </Card>
  );
}
