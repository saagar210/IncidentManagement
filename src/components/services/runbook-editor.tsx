import { useState } from "react";
import { MarkdownEditor } from "@/components/ui/markdown-editor";
import { MarkdownViewer } from "@/components/ui/markdown-viewer";
import { Button } from "@/components/ui/button";
import { Pencil, Eye } from "lucide-react";

interface RunbookEditorProps {
  value: string;
  onChange: (value: string) => void;
  readOnly?: boolean;
}

export function RunbookEditor({ value, onChange, readOnly }: RunbookEditorProps) {
  const [editing, setEditing] = useState(false);

  if (readOnly || !editing) {
    return (
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-medium">Runbook</h3>
          {!readOnly && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setEditing(true)}
            >
              <Pencil className="mr-1 h-3 w-3" />
              Edit
            </Button>
          )}
        </div>
        <div className="rounded-md border p-4">
          <MarkdownViewer content={value} />
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">Runbook</h3>
        <Button variant="ghost" size="sm" onClick={() => setEditing(false)}>
          <Eye className="mr-1 h-3 w-3" />
          Preview
        </Button>
      </div>
      <MarkdownEditor
        value={value}
        onChange={onChange}
        placeholder="Document response procedures, escalation paths, troubleshooting steps..."
        height={300}
      />
    </div>
  );
}
