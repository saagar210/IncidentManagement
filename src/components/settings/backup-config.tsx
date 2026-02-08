import { useState } from "react";
import { Database, FolderOpen, RefreshCw } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { useBackups, useCreateBackup } from "@/hooks/use-backup";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { toast } from "@/components/ui/use-toast";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export function BackupConfig() {
  const [backupDir, setBackupDir] = useState("");
  const { data: backups, refetch } = useBackups(backupDir);
  const createBackup = useCreateBackup();

  const handleBrowse = async () => {
    const selected = await open({ directory: true });
    if (selected) {
      setBackupDir(selected);
    }
  };

  const handleBackup = async () => {
    if (!backupDir) {
      toast({
        title: "Select a backup directory first",
        variant: "destructive",
      });
      return;
    }

    try {
      // Default SQLite path for Tauri apps
      const dbPath = "incidents.db";
      const path = await createBackup.mutateAsync({
        dbPath,
        backupDir,
      });
      toast({
        title: "Backup created",
        description: `Saved to ${path}`,
      });
      refetch();
    } catch (err) {
      toast({
        title: "Backup failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  return (
    <div className="space-y-4">
      <div>
        <h2 className="text-lg font-semibold">Database Backup</h2>
        <p className="text-sm text-muted-foreground">
          Create a copy of your SQLite database for safekeeping.
        </p>
      </div>

      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-base">
            <Database className="h-4 w-4" />
            Backup Configuration
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Backup Directory</Label>
            <div className="flex gap-2">
              <Input
                value={backupDir}
                onChange={(e) => setBackupDir(e.target.value)}
                placeholder="Select a directory..."
                className="flex-1"
              />
              <Button variant="outline" onClick={handleBrowse}>
                <FolderOpen className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <Button
            onClick={handleBackup}
            disabled={!backupDir || createBackup.isPending}
          >
            {createBackup.isPending ? (
              <RefreshCw className="mr-1 h-4 w-4 animate-spin" />
            ) : (
              <Database className="mr-1 h-4 w-4" />
            )}
            {createBackup.isPending ? "Backing up..." : "Create Backup Now"}
          </Button>
        </CardContent>
      </Card>

      {/* Existing backups */}
      {backups && backups.length > 0 && (
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">Existing Backups</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-1">
              {backups.map((b) => (
                <div
                  key={b.path}
                  className="flex items-center justify-between rounded border px-3 py-2"
                >
                  <div>
                    <span className="text-sm font-medium">{b.name}</span>
                    <span className="ml-2 text-xs text-muted-foreground">
                      {formatBytes(b.size_bytes)}
                    </span>
                  </div>
                  <span className="text-xs text-muted-foreground">
                    {b.created_at
                      ? new Date(b.created_at).toLocaleString()
                      : ""}
                  </span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
