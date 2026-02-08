import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import type { ImportPreview as ImportPreviewData } from "@/hooks/use-import";

interface ImportPreviewProps {
  preview: ImportPreviewData;
}

const STATUS_STYLES: Record<string, string> = {
  ready: "bg-green-500/10 text-green-600 border-green-500/20",
  warning: "bg-yellow-500/10 text-yellow-600 border-yellow-500/20",
  error: "bg-red-500/10 text-red-500 border-red-500/20",
};

export function ImportPreviewTable({ preview }: ImportPreviewProps) {
  return (
    <div className="space-y-4">
      {/* Summary counts */}
      <div className="flex gap-4 text-sm">
        <span className="text-green-600 font-medium">
          {preview.ready_count} ready
        </span>
        {preview.warning_count > 0 && (
          <span className="text-yellow-600 font-medium">
            {preview.warning_count} warnings
          </span>
        )}
        {preview.error_count > 0 && (
          <span className="text-red-500 font-medium">
            {preview.error_count} errors
          </span>
        )}
      </div>

      {/* Preview table */}
      <div className="max-h-[300px] overflow-y-auto rounded border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-16">Status</TableHead>
              <TableHead>Title</TableHead>
              <TableHead>Service</TableHead>
              <TableHead>Severity</TableHead>
              <TableHead>Started At</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {preview.incidents.map((row, idx) => (
              <TableRow key={idx}>
                <TableCell>
                  <Badge className={STATUS_STYLES[row.row_status] ?? ""}>
                    {row.row_status}
                  </Badge>
                </TableCell>
                <TableCell className="font-medium max-w-[200px] truncate">
                  {row.title || "(empty)"}
                </TableCell>
                <TableCell>{row.service_name || "(empty)"}</TableCell>
                <TableCell>{row.severity || "(empty)"}</TableCell>
                <TableCell>{row.started_at || "(empty)"}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      {/* Warnings list */}
      {preview.warnings.length > 0 && (
        <div className="space-y-1">
          <p className="text-sm font-medium text-yellow-600">Warnings:</p>
          <ul className="text-xs text-muted-foreground space-y-0.5 max-h-[100px] overflow-y-auto">
            {preview.warnings.map((w, idx) => (
              <li key={idx}>
                Row {w.row}: {w.message}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
