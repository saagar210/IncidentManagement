import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { Attachment } from "@/types/attachments";

export function useAttachments(incidentId: string | undefined) {
  return useQuery({
    queryKey: ["attachments", incidentId],
    queryFn: () =>
      tauriInvoke<Attachment[]>("list_attachments", {
        incidentId: incidentId as string,
      }),
    enabled: !!incidentId,
  });
}

export function useUploadAttachment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      incidentId,
      sourcePath,
      filename,
    }: {
      incidentId: string;
      sourcePath: string;
      filename: string;
    }) =>
      tauriInvoke<Attachment>("upload_attachment", {
        incidentId,
        sourcePath,
        filename,
      }),
    onSuccess: (_, vars) =>
      qc.invalidateQueries({ queryKey: ["attachments", vars.incidentId] }),
  });
}

export function useDeleteAttachment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id }: { id: string; incidentId: string }) =>
      tauriInvoke<void>("delete_attachment", { id }),
    onSuccess: (_, vars) =>
      qc.invalidateQueries({ queryKey: ["attachments", vars.incidentId] }),
  });
}
