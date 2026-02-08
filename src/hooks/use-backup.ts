import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";

export interface BackupInfo {
  name: string;
  path: string;
  size_bytes: number;
  created_at: string;
}

export function useBackups(backupDir: string) {
  return useQuery({
    queryKey: ["backups", backupDir],
    queryFn: () =>
      tauriInvoke<BackupInfo[]>("list_backups", { backupDir }),
    enabled: !!backupDir,
  });
}

export function useCreateBackup() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (params: { backupDir: string }) =>
      tauriInvoke<string>("create_backup", params),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["backups"] });
    },
  });
}
