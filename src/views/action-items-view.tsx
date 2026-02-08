import { useState, useMemo } from "react";
import { Link } from "react-router-dom";
import { CheckCircle2, Circle, Clock, ArrowUpRight } from "lucide-react";
import { format, isPast, isWithinInterval, addDays, formatDistanceToNow } from "date-fns";
import { useActionItems, useUpdateActionItem } from "@/hooks/use-incidents";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Select } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ACTION_ITEM_STATUSES } from "@/lib/constants";
import type { ActionItem } from "@/types/incident";

const STATUS_CYCLE: Record<string, string> = {
  Open: "In-Progress",
  "In-Progress": "Done",
  Done: "Open",
};

const STATUS_BADGE_COLORS: Record<string, string> = {
  Open: "bg-blue-500/10 text-blue-500 border-blue-500/20",
  "In-Progress": "bg-yellow-500/10 text-yellow-600 border-yellow-500/20",
  Done: "bg-green-500/10 text-green-600 border-green-500/20",
};

const STATUS_ICONS: Record<string, React.ReactNode> = {
  Open: <Circle className="h-3 w-3" />,
  "In-Progress": <Clock className="h-3 w-3" />,
  Done: <CheckCircle2 className="h-3 w-3" />,
};

function formatDueDate(dateStr: string | null): { text: string; className: string } {
  if (!dateStr) return { text: "--", className: "text-muted-foreground" };

  try {
    const date = new Date(dateStr);
    const now = new Date();
    const formatted = format(date, "MMM d, yyyy");

    if (isPast(date) && date < now) {
      return { text: formatted, className: "text-red-500 font-medium" };
    }

    if (isWithinInterval(date, { start: now, end: addDays(now, 3) })) {
      return { text: formatted, className: "text-amber-500 font-medium" };
    }

    return { text: formatted, className: "" };
  } catch {
    return { text: dateStr, className: "" };
  }
}

export function ActionItemsView() {
  const [statusFilter, setStatusFilter] = useState<string>("");
  const [overdueOnly, setOverdueOnly] = useState(false);
  const [sortBy, setSortBy] = useState<"due_date" | "created_at" | "status">("due_date");

  const { data: actionItems, isLoading } = useActionItems();
  const updateActionItem = useUpdateActionItem();

  const filteredAndSorted = useMemo(() => {
    if (!actionItems) return [];

    let items = [...actionItems];

    // Filter by status
    if (statusFilter) {
      items = items.filter((item: ActionItem) => item.status === statusFilter);
    }

    // Filter overdue only
    if (overdueOnly) {
      const now = new Date();
      items = items.filter(
        (item: ActionItem) =>
          item.due_date !== null &&
          new Date(item.due_date) < now &&
          item.status !== "Done"
      );
    }

    // Sort
    items.sort((a: ActionItem, b: ActionItem) => {
      if (sortBy === "due_date") {
        if (!a.due_date && !b.due_date) return 0;
        if (!a.due_date) return 1;
        if (!b.due_date) return -1;
        return new Date(a.due_date).getTime() - new Date(b.due_date).getTime();
      }
      if (sortBy === "created_at") {
        return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
      }
      // status sort: Open first, then In-Progress, then Done
      const order: Record<string, number> = { Open: 0, "In-Progress": 1, Done: 2 };
      return (order[a.status] ?? 3) - (order[b.status] ?? 3);
    });

    return items;
  }, [actionItems, statusFilter, overdueOnly, sortBy]);

  function handleCycleStatus(item: ActionItem) {
    const nextStatus = STATUS_CYCLE[item.status] ?? "Open";
    updateActionItem.mutate({
      id: item.id,
      item: { status: nextStatus },
    });
  }

  return (
    <div className="space-y-4 p-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Action Items</h1>
      </div>

      {/* Filter bar */}
      <div className="flex flex-wrap items-center gap-3">
        <Select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          className="w-40"
        >
          <option value="">All Status</option>
          {ACTION_ITEM_STATUSES.map((s) => (
            <option key={s} value={s}>
              {s}
            </option>
          ))}
        </Select>
        <Button
          variant={overdueOnly ? "secondary" : "outline"}
          size="sm"
          onClick={() => setOverdueOnly((prev) => !prev)}
        >
          <Clock className="h-4 w-4" />
          Overdue Only
        </Button>
        <Select
          value={sortBy}
          onChange={(e) => setSortBy(e.target.value as "due_date" | "created_at" | "status")}
          className="w-36"
        >
          <option value="due_date">Due Date</option>
          <option value="created_at">Created</option>
          <option value="status">Status</option>
        </Select>
      </div>

      {/* Content */}
      {isLoading ? (
        <div className="space-y-2">
          {Array.from({ length: 8 }).map((_, i) => (
            <Skeleton key={i} className="h-10 w-full" />
          ))}
        </div>
      ) : filteredAndSorted.length === 0 ? (
        <div className="flex h-64 items-center justify-center text-muted-foreground">
          <p>No action items found. Action items are created from incident detail pages.</p>
        </div>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Status</TableHead>
              <TableHead>Title</TableHead>
              <TableHead>Owner</TableHead>
              <TableHead>Due Date</TableHead>
              <TableHead>Incident</TableHead>
              <TableHead>Created</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {filteredAndSorted.map((item: ActionItem) => {
              const dueDate = formatDueDate(item.due_date);
              return (
                <TableRow key={item.id}>
                  <TableCell>
                    <Badge
                      variant="outline"
                      className={`cursor-pointer gap-1 ${STATUS_BADGE_COLORS[item.status] ?? ""}`}
                      onClick={() => handleCycleStatus(item)}
                    >
                      {STATUS_ICONS[item.status]}
                      {item.status}
                    </Badge>
                  </TableCell>
                  <TableCell className="font-medium max-w-[250px] truncate">
                    {item.title}
                  </TableCell>
                  <TableCell>
                    {item.owner ? (
                      item.owner
                    ) : (
                      <span className="text-muted-foreground">Unassigned</span>
                    )}
                  </TableCell>
                  <TableCell className={`whitespace-nowrap text-sm ${dueDate.className}`}>
                    {dueDate.text}
                  </TableCell>
                  <TableCell>
                    {(item as ActionItem & { incident_title?: string }).incident_title ? (
                      <Link
                        to={`/incidents/${item.incident_id}`}
                        className="inline-flex items-center gap-1 text-sm text-primary hover:underline"
                        onClick={(e) => e.stopPropagation()}
                      >
                        {(item as ActionItem & { incident_title?: string }).incident_title}
                        <ArrowUpRight className="h-3 w-3" />
                      </Link>
                    ) : (
                      <Link
                        to={`/incidents/${item.incident_id}`}
                        className="inline-flex items-center gap-1 text-sm text-primary hover:underline"
                        onClick={(e) => e.stopPropagation()}
                      >
                        View Incident
                        <ArrowUpRight className="h-3 w-3" />
                      </Link>
                    )}
                  </TableCell>
                  <TableCell className="whitespace-nowrap text-sm text-muted-foreground">
                    {formatDistanceToNow(new Date(item.created_at), { addSuffix: true })}
                  </TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      )}
    </div>
  );
}
