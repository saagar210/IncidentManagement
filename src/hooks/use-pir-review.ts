import { useMutation, useQuery } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";

type PirBrief = {
  markdown: string;
};

export function usePirBriefMarkdown() {
  return useMutation({
    mutationFn: (incidentId: string) =>
      tauriInvoke<PirBrief>("generate_pir_brief_markdown", { incidentId }),
  });
}

export function useGeneratePirBriefFile() {
  return useMutation({
    mutationFn: ({ incidentId, format }: { incidentId: string; format: "docx" | "pdf" }) =>
      tauriInvoke<string>("generate_pir_brief_file", { incidentId, format }),
  });
}

type PirInsightCount = { label: string; count: number };

type PirReviewInsights = {
  top_factor_categories: PirInsightCount[];
  top_factor_descriptions: PirInsightCount[];
  external_root_no_action_items_justified: number;
};

export function usePirReviewInsights() {
  return useQuery({
    queryKey: ["pir-review-insights"],
    queryFn: () => tauriInvoke<PirReviewInsights>("get_pir_review_insights"),
    staleTime: 60_000,
  });
}

