import { FileSpreadsheet, FileJson } from "lucide-react";
import { save } from "@tauri-apps/plugin-dialog";
import { copyFile } from "@tauri-apps/plugin-fs";
import { useExportCsv, useExportJson } from "@/hooks/use-export";
import { Button } from "@/components/ui/button";
import { toast } from "@/components/ui/use-toast";
import { tauriInvoke } from "@/lib/tauri";
import type { IncidentFilters } from "@/types/incident";

interface ExportMenuProps {
  filters: IncidentFilters;
}

export function ExportMenu({ filters }: ExportMenuProps) {
  const exportCsv = useExportCsv();
  const exportJson = useExportJson();

  const handleExport = async (format: "csv" | "json") => {
    try {
      const mutation = format === "csv" ? exportCsv : exportJson;
      const ext = format;
      const tempPath = await mutation.mutateAsync(filters);

      const savePath = await save({
        defaultPath: `incidents_export.${ext}`,
        filters: [
          {
            name: `${ext.toUpperCase()} Files`,
            extensions: [ext],
          },
        ],
      });

      if (savePath) {
        await copyFile(tempPath, savePath);
        // Best-effort cleanup of the backend temp file.
        await tauriInvoke("delete_temp_file", { tempPath });
        toast({
          title: "Export complete",
          description: `Saved to ${savePath}`,
        });
      } else {
        // User cancelled: best-effort cleanup of the backend temp file.
        await tauriInvoke("delete_temp_file", { tempPath });
      }
    } catch (err) {
      toast({
        title: "Export failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  return (
    <div className="flex items-center gap-1">
      <Button
        size="sm"
        variant="outline"
        onClick={() => handleExport("csv")}
        disabled={exportCsv.isPending}
        title="Export as CSV"
      >
        <FileSpreadsheet className="h-3.5 w-3.5" />
        <span className="hidden sm:inline">CSV</span>
      </Button>
      <Button
        size="sm"
        variant="outline"
        onClick={() => handleExport("json")}
        disabled={exportJson.isPending}
        title="Export as JSON"
      >
        <FileJson className="h-3.5 w-3.5" />
        <span className="hidden sm:inline">JSON</span>
      </Button>
    </div>
  );
}
