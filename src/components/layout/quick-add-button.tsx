import { Plus } from "lucide-react";
import { Button } from "@/components/ui/button";

interface QuickAddButtonProps {
  onQuickAdd: () => void;
}

export function QuickAddButton({ onQuickAdd }: QuickAddButtonProps) {
  return (
    <div className="fixed bottom-6 right-6 z-50" title="New Incident (⌘N)">
      <Button
        size="icon"
        aria-label="New incident"
        className="h-12 w-12 rounded-full shadow-lg"
        onClick={onQuickAdd}
      >
        <Plus className="h-5 w-5" />
      </Button>
    </div>
  );
}
