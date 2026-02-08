import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Command } from "cmdk";
import {
  BarChart3,
  AlertTriangle,
  FileText,
  Settings,
  Plus,
  Search,
  Sun,
  Moon,
  Monitor,
  Trash2,
} from "lucide-react";
import { useSearchIncidents } from "@/hooks/use-incidents";
import { useTheme, type Theme } from "@/hooks/use-theme";
import { Dialog, DialogContent } from "@/components/ui/dialog";

interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function CommandPalette({ open, onOpenChange }: CommandPaletteProps) {
  const navigate = useNavigate();
  const { setTheme } = useTheme();
  const [search, setSearch] = useState("");
  const { data: searchResults } = useSearchIncidents(search);

  // Reset search when closing
  useEffect(() => {
    if (!open) setSearch("");
  }, [open]);

  const runAction = useCallback(
    (action: () => void) => {
      action();
      onOpenChange(false);
    },
    [onOpenChange]
  );

  const setThemeAction = useCallback(
    (theme: Theme) => {
      setTheme(theme);
      onOpenChange(false);
    },
    [setTheme, onOpenChange]
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="overflow-hidden p-0 shadow-lg max-w-[520px]">
        <Command
          className="[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:font-medium [&_[cmdk-group-heading]]:text-muted-foreground [&_[cmdk-group]]:px-2 [&_[cmdk-item]]:px-2 [&_[cmdk-item]]:py-2 [&_[cmdk-input-wrapper]_svg]:h-5 [&_[cmdk-input-wrapper]_svg]:w-5 [&_[cmdk-input]]:h-12"
          loop
        >
          <div className="flex items-center border-b px-3">
            <Search className="mr-2 h-4 w-4 shrink-0 opacity-50" />
            <Command.Input
              placeholder="Search incidents, navigate, or run actions..."
              value={search}
              onValueChange={setSearch}
              className="flex h-11 w-full rounded-md bg-transparent py-3 text-sm outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50"
            />
          </div>
          <Command.List className="max-h-[400px] overflow-y-auto p-2">
            <Command.Empty className="py-6 text-center text-sm text-muted-foreground">
              No results found.
            </Command.Empty>

            {/* Live incident search results */}
            {search && searchResults && searchResults.length > 0 && (
              <Command.Group heading="Incidents">
                {searchResults.slice(0, 5).map((inc) => (
                  <Command.Item
                    key={inc.id}
                    value={`incident-${inc.title}`}
                    onSelect={() =>
                      runAction(() => navigate(`/incidents/${inc.id}`))
                    }
                    className="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer aria-selected:bg-accent"
                  >
                    <AlertTriangle className="h-4 w-4 text-muted-foreground" />
                    <span className="flex-1 truncate">{inc.title}</span>
                    <span className="text-xs text-muted-foreground">
                      {inc.severity}
                    </span>
                  </Command.Item>
                ))}
              </Command.Group>
            )}

            <Command.Group heading="Navigation">
              <Command.Item
                onSelect={() => runAction(() => navigate("/dashboard"))}
                className="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer aria-selected:bg-accent"
              >
                <BarChart3 className="h-4 w-4 text-muted-foreground" />
                Dashboard
                <kbd className="ml-auto text-[10px] text-muted-foreground/60">
                  {"\u2318"}1
                </kbd>
              </Command.Item>
              <Command.Item
                onSelect={() => runAction(() => navigate("/incidents"))}
                className="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer aria-selected:bg-accent"
              >
                <AlertTriangle className="h-4 w-4 text-muted-foreground" />
                Incidents
                <kbd className="ml-auto text-[10px] text-muted-foreground/60">
                  {"\u2318"}2
                </kbd>
              </Command.Item>
              <Command.Item
                onSelect={() => runAction(() => navigate("/reports"))}
                className="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer aria-selected:bg-accent"
              >
                <FileText className="h-4 w-4 text-muted-foreground" />
                Reports
                <kbd className="ml-auto text-[10px] text-muted-foreground/60">
                  {"\u2318"}3
                </kbd>
              </Command.Item>
              <Command.Item
                onSelect={() => runAction(() => navigate("/settings"))}
                className="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer aria-selected:bg-accent"
              >
                <Settings className="h-4 w-4 text-muted-foreground" />
                Settings
                <kbd className="ml-auto text-[10px] text-muted-foreground/60">
                  {"\u2318"}4
                </kbd>
              </Command.Item>
              <Command.Item
                onSelect={() => runAction(() => navigate("/trash"))}
                className="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer aria-selected:bg-accent"
              >
                <Trash2 className="h-4 w-4 text-muted-foreground" />
                Trash
              </Command.Item>
            </Command.Group>

            <Command.Group heading="Actions">
              <Command.Item
                onSelect={() =>
                  runAction(() => navigate("/incidents/new"))
                }
                className="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer aria-selected:bg-accent"
              >
                <Plus className="h-4 w-4 text-muted-foreground" />
                New Incident
                <kbd className="ml-auto text-[10px] text-muted-foreground/60">
                  {"\u2318"}N
                </kbd>
              </Command.Item>
            </Command.Group>

            <Command.Group heading="Theme">
              <Command.Item
                onSelect={() => setThemeAction("light")}
                className="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer aria-selected:bg-accent"
              >
                <Sun className="h-4 w-4 text-muted-foreground" />
                Light Mode
              </Command.Item>
              <Command.Item
                onSelect={() => setThemeAction("dark")}
                className="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer aria-selected:bg-accent"
              >
                <Moon className="h-4 w-4 text-muted-foreground" />
                Dark Mode
              </Command.Item>
              <Command.Item
                onSelect={() => setThemeAction("system")}
                className="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer aria-selected:bg-accent"
              >
                <Monitor className="h-4 w-4 text-muted-foreground" />
                System Theme
              </Command.Item>
            </Command.Group>
          </Command.List>
        </Command>
      </DialogContent>
    </Dialog>
  );
}
