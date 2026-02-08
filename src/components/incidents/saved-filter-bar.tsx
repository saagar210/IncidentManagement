import { useState } from "react";
import { Bookmark, Plus, Trash2, Star } from "lucide-react";
import {
  useSavedFilters,
  useCreateSavedFilter,
  useDeleteSavedFilter,
} from "@/hooks/use-saved-filters";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { toast } from "@/components/ui/use-toast";
import type { IncidentFilters } from "@/types/incident";

interface SavedFilterBarProps {
  currentFilters: IncidentFilters;
  onApplyFilter: (filters: IncidentFilters) => void;
}

export function SavedFilterBar({ currentFilters, onApplyFilter }: SavedFilterBarProps) {
  const { data: savedFilters } = useSavedFilters();
  const createMutation = useCreateSavedFilter();
  const deleteMutation = useDeleteSavedFilter();
  const [showSave, setShowSave] = useState(false);
  const [filterName, setFilterName] = useState("");

  const handleSave = async () => {
    const name = filterName.trim();
    if (!name) return;

    try {
      await createMutation.mutateAsync({
        name,
        filters: JSON.stringify(currentFilters),
      });
      setFilterName("");
      setShowSave(false);
      toast({ title: `Filter "${name}" saved` });
    } catch (err) {
      toast({ title: "Failed to save filter", description: String(err), variant: "destructive" });
    }
  };

  const handleApply = (filtersJson: string) => {
    try {
      const parsed = JSON.parse(filtersJson) as IncidentFilters;
      onApplyFilter(parsed);
    } catch {
      toast({ title: "Invalid filter data", variant: "destructive" });
    }
  };

  const handleDelete = async (id: string, name: string) => {
    try {
      await deleteMutation.mutateAsync(id);
      toast({ title: `Filter "${name}" deleted` });
    } catch (err) {
      toast({ title: "Failed to delete filter", description: String(err), variant: "destructive" });
    }
  };

  const hasFilters = Object.values(currentFilters).some((v) => v !== undefined && v !== "");

  return (
    <div className="flex flex-wrap items-center gap-2">
      <Bookmark className="h-4 w-4 text-muted-foreground" />

      {savedFilters?.map((sf) => (
        <div key={sf.id} className="group flex items-center gap-0.5">
          <Badge
            variant="outline"
            className="cursor-pointer hover:bg-accent"
            onClick={() => handleApply(sf.filters)}
          >
            {sf.is_default && <Star className="mr-1 h-3 w-3 fill-yellow-500 text-yellow-500" />}
            {sf.name}
          </Badge>
          <button
            onClick={() => handleDelete(sf.id, sf.name)}
            className="hidden rounded p-0.5 text-muted-foreground hover:text-destructive group-hover:block"
          >
            <Trash2 className="h-3 w-3" />
          </button>
        </div>
      ))}

      {showSave ? (
        <div className="flex items-center gap-1">
          <Input
            value={filterName}
            onChange={(e) => setFilterName(e.target.value)}
            placeholder="Filter name..."
            className="h-7 w-32 text-xs"
            onKeyDown={(e) => {
              if (e.key === "Enter") handleSave();
              if (e.key === "Escape") setShowSave(false);
            }}
            autoFocus
          />
          <Button size="sm" variant="ghost" className="h-7 px-2 text-xs" onClick={handleSave}>
            Save
          </Button>
        </div>
      ) : (
        hasFilters && (
          <Button
            size="sm"
            variant="ghost"
            className="h-7 gap-1 px-2 text-xs"
            onClick={() => setShowSave(true)}
          >
            <Plus className="h-3 w-3" />
            Save Filter
          </Button>
        )
      )}
    </div>
  );
}
