import { Paperclip, Trash2, Image, FileText, File } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useAttachments, useUploadAttachment, useDeleteAttachment } from "@/hooks/use-attachments";
import { toast } from "@/components/ui/use-toast";
import type { Attachment } from "@/types/attachments";

interface AttachmentListProps {
  incidentId: string | undefined;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function FileIcon({ mime }: { mime: string }) {
  if (mime.startsWith("image/")) return <Image className="h-4 w-4 text-blue-500" />;
  if (mime.includes("pdf") || mime.includes("document"))
    return <FileText className="h-4 w-4 text-red-500" />;
  return <File className="h-4 w-4 text-muted-foreground" />;
}

export function AttachmentList({ incidentId }: AttachmentListProps) {
  const { data: attachments } = useAttachments(incidentId);
  const uploadMutation = useUploadAttachment();
  const deleteMutation = useDeleteAttachment();

  if (!incidentId) return null;

  const handleUpload = async () => {
    try {
      const selected = await open({ multiple: false });
      if (!selected) return;

      const path = selected;
      const filename = path.split("/").pop() ?? "file";

      await uploadMutation.mutateAsync({
        incidentId,
        sourcePath: path,
        filename,
      });
      toast({ title: "File uploaded", description: filename });
    } catch (err) {
      toast({
        title: "Upload failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleDelete = async (att: Attachment) => {
    const confirmed = window.confirm(`Delete "${att.filename}"?`);
    if (!confirmed) return;
    try {
      await deleteMutation.mutateAsync({ id: att.id, incidentId });
    } catch (err) {
      toast({
        title: "Delete failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle className="text-base">Attachments</CardTitle>
        <Button size="sm" variant="outline" onClick={handleUpload} disabled={uploadMutation.isPending}>
          <Paperclip className="h-4 w-4" />
          {uploadMutation.isPending ? "Uploading..." : "Attach File"}
        </Button>
      </CardHeader>
      <CardContent>
        {!attachments || attachments.length === 0 ? (
          <p className="text-sm text-muted-foreground">No attachments yet.</p>
        ) : (
          <div className="space-y-2">
            {attachments.map((att) => (
              <div
                key={att.id}
                className="flex items-center justify-between rounded border px-3 py-2"
              >
                <div className="flex items-center gap-2 min-w-0">
                  <FileIcon mime={att.mime_type} />
                  <span className="text-sm font-medium truncate">{att.filename}</span>
                  <span className="text-xs text-muted-foreground shrink-0">
                    {formatBytes(att.size_bytes)}
                  </span>
                </div>
                <Button
                  size="icon"
                  variant="ghost"
                  onClick={() => handleDelete(att)}
                  disabled={deleteMutation.isPending}
                >
                  <Trash2 className="h-4 w-4 text-destructive" />
                </Button>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
