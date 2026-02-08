import { useQuery, useMutation } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { ReportConfig, DiscussionPoint } from "@/types/reports";

export function useGenerateReport() {
  return useMutation({
    mutationFn: (config: ReportConfig) =>
      tauriInvoke<string>("generate_report", { config }),
  });
}

export function useSaveReport() {
  return useMutation({
    mutationFn: ({ tempPath, savePath }: { tempPath: string; savePath: string }) =>
      tauriInvoke<void>("save_report", { tempPath, savePath }),
  });
}

export function useDiscussionPoints(quarterId: string | null) {
  return useQuery({
    queryKey: ["discussion-points", quarterId],
    queryFn: () =>
      tauriInvoke<DiscussionPoint[]>("generate_discussion_points", {
        quarterId: quarterId!,
      }),
    enabled: !!quarterId,
  });
}
