import { useQuery, useMutation } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { AiStatus, SimilarIncident } from "@/types/ai";

export function useAiStatus() {
  return useQuery({
    queryKey: ["ai-status"],
    queryFn: () => tauriInvoke<AiStatus>("get_ai_status"),
    staleTime: 30_000,
    refetchInterval: 60_000,
  });
}

export function useCheckAiHealth() {
  return useMutation({
    mutationFn: () => tauriInvoke<boolean>("check_ai_health"),
  });
}

export function useAiSummarize() {
  return useMutation({
    mutationFn: (params: {
      title: string;
      severity: string;
      status: string;
      service: string;
      rootCause: string;
      resolution: string;
      notes: string;
    }) =>
      tauriInvoke<string>("ai_summarize_incident", {
        title: params.title,
        severity: params.severity,
        status: params.status,
        service: params.service,
        rootCause: params.rootCause,
        resolution: params.resolution,
        notes: params.notes,
      }),
  });
}

export function useAiStakeholderUpdate() {
  return useMutation({
    mutationFn: (params: {
      title: string;
      severity: string;
      status: string;
      service: string;
      impact: string;
      notes: string;
    }) =>
      tauriInvoke<string>("ai_stakeholder_update", {
        title: params.title,
        severity: params.severity,
        status: params.status,
        service: params.service,
        impact: params.impact,
        notes: params.notes,
      }),
  });
}

export function useAiPostmortemDraft() {
  return useMutation({
    mutationFn: (params: {
      title: string;
      severity: string;
      service: string;
      rootCause: string;
      resolution: string;
      lessons: string;
      contributingFactors: string[];
    }) =>
      tauriInvoke<string>("ai_postmortem_draft", {
        title: params.title,
        severity: params.severity,
        service: params.service,
        rootCause: params.rootCause,
        resolution: params.resolution,
        lessons: params.lessons,
        contributingFactors: params.contributingFactors,
      }),
  });
}

export function useSimilarIncidents(
  query: string,
  excludeId?: string,
  limit = 5
) {
  return useQuery({
    queryKey: ["similar-incidents", query, excludeId],
    queryFn: () =>
      tauriInvoke<SimilarIncident[]>("find_similar_incidents", {
        query,
        excludeId,
        limit,
      }),
    enabled: query.trim().length >= 3,
    staleTime: 60_000,
  });
}

export interface ServiceTrend {
  service_id: string;
  service_name: string;
  trend_type: string;
  message: string;
  incident_count_current: number;
  incident_count_previous: number;
}

export function useServiceTrends() {
  return useQuery({
    queryKey: ["service-trends"],
    queryFn: () => tauriInvoke<ServiceTrend[]>("detect_service_trends"),
    staleTime: 300_000, // 5 min
  });
}
