import { useState } from "react";
import { Plus, Trash2, Star } from "lucide-react";
import {
  useContributingFactors,
  useCreateContributingFactor,
  useDeleteContributingFactor,
} from "@/hooks/use-postmortems";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { toast } from "@/components/ui/use-toast";
import {
  CONTRIBUTING_FACTOR_CATEGORIES,
  type ContributingFactorCategory,
} from "@/types/postmortem";

interface ContributingFactorsFormProps {
  incidentId: string;
}

const CATEGORY_COLORS: Record<ContributingFactorCategory, string> = {
  Process: "bg-blue-500/10 text-blue-600 border-blue-500/20",
  Tooling: "bg-purple-500/10 text-purple-600 border-purple-500/20",
  Communication: "bg-yellow-500/10 text-yellow-600 border-yellow-500/20",
  "Human Factors": "bg-orange-500/10 text-orange-600 border-orange-500/20",
  External: "bg-gray-500/10 text-gray-600 border-gray-500/20",
};

export function ContributingFactorsForm({
  incidentId,
}: ContributingFactorsFormProps) {
  const { data: factors } = useContributingFactors(incidentId);
  const createMutation = useCreateContributingFactor();
  const deleteMutation = useDeleteContributingFactor();

  const [newCategory, setNewCategory] = useState<ContributingFactorCategory>(
    CONTRIBUTING_FACTOR_CATEGORIES[0]
  );
  const [newDescription, setNewDescription] = useState("");
  const [isRoot, setIsRoot] = useState(false);

  const handleAdd = async () => {
    if (!newDescription.trim()) return;
    try {
      await createMutation.mutateAsync({
        incident_id: incidentId,
        category: newCategory,
        description: newDescription.trim(),
        is_root: isRoot,
      });
      setNewDescription("");
      setIsRoot(false);
    } catch (err) {
      toast({
        title: "Failed to add factor",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteMutation.mutateAsync(id);
    } catch (err) {
      toast({
        title: "Failed to delete factor",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-base">Contributing Factors</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {/* Existing factors */}
        {factors && factors.length > 0 && (
          <div className="space-y-2">
            {factors.map((f) => (
              <div
                key={f.id}
                className="flex items-start justify-between gap-2 rounded-md border p-2"
              >
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <Badge
                      variant="outline"
                      className={CATEGORY_COLORS[f.category as ContributingFactorCategory] ?? ""}
                    >
                      {f.category}
                    </Badge>
                    {f.is_root && (
                      <Star className="h-3.5 w-3.5 fill-yellow-500 text-yellow-500" />
                    )}
                  </div>
                  <p className="mt-1 text-sm">{f.description}</p>
                </div>
                <Button
                  size="sm"
                  variant="ghost"
                  className="h-7 w-7 p-0 text-muted-foreground hover:text-destructive"
                  onClick={() => handleDelete(f.id)}
                >
                  <Trash2 className="h-3.5 w-3.5" />
                </Button>
              </div>
            ))}
          </div>
        )}

        {/* Add new factor */}
        <div className="flex flex-wrap items-end gap-2">
          <div className="flex-1 min-w-[200px]">
            <Input
              placeholder="Describe the contributing factor..."
              value={newDescription}
              onChange={(e) => setNewDescription(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleAdd();
              }}
            />
          </div>
          <Select
            value={newCategory}
            onChange={(e) =>
              setNewCategory(e.target.value as ContributingFactorCategory)
            }
            className="w-40"
          >
            {CONTRIBUTING_FACTOR_CATEGORIES.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </Select>
          <label className="flex items-center gap-1 text-xs">
            <input
              type="checkbox"
              checked={isRoot}
              onChange={(e) => setIsRoot(e.target.checked)}
              className="h-3.5 w-3.5 rounded border-input"
            />
            Root
          </label>
          <Button size="sm" onClick={handleAdd} disabled={!newDescription.trim()}>
            <Plus className="h-3.5 w-3.5" />
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
