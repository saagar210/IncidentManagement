import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { ReportConfig, DiscussionPoint, ReportHistoryEntry } from "@/types/reports";

export function useGenerateReport() {
  return useMutation({
    mutationFn: (config: ReportConfig) =>
      tauriInvoke<string>("generate_report", { config }),
  });
}

export function useSaveReport() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      tempPath,
      savePath,
      title,
      quarterId,
      configJson,
    }: {
      tempPath: string;
      savePath: string;
      title: string;
      quarterId?: string | null;
      configJson?: string;
    }) =>
      tauriInvoke<ReportHistoryEntry>("save_report", {
        tempPath,
        savePath,
        title,
        quarterId: quarterId ?? null,
        configJson: configJson ?? null,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["report-history"] });
    },
  });
}

export function useDiscussionPoints(quarterId: string | null) {
  return useQuery({
    queryKey: ["discussion-points", quarterId],
    queryFn: () =>
      tauriInvoke<DiscussionPoint[]>("generate_discussion_points", {
        quarterId: quarterId as string,
      }),
    enabled: quarterId !== null && quarterId.length > 0,
  });
}

export function useReportHistory() {
  return useQuery({
    queryKey: ["report-history"],
    queryFn: () => tauriInvoke<ReportHistoryEntry[]>("list_report_history"),
  });
}

export function useDeleteReportHistory() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_report_history_entry", { id }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["report-history"] });
    },
  });
}

export function useGenerateNarrative() {
  return useMutation({
    mutationFn: (quarterId: string) =>
      tauriInvoke<string>("generate_narrative", { quarterId }),
  });
}
