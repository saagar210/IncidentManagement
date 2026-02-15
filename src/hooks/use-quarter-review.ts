import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type {
  MetricDefinition,
  QuarterFinalizationStatus,
  QuarterOverride,
  QuarterReadinessReport,
} from "@/types/quarter-review";

export function useQuarterReadiness(quarterId: string | null) {
  return useQuery({
    queryKey: ["quarter-readiness", quarterId],
    queryFn: () =>
      tauriInvoke<QuarterReadinessReport>("get_quarter_readiness", {
        quarterId,
      }),
    enabled: !!quarterId,
    staleTime: 30000,
  });
}

export function useMetricGlossary() {
  return useQuery({
    queryKey: ["metric-glossary"],
    queryFn: () => tauriInvoke<MetricDefinition[]>("get_metric_glossary"),
    staleTime: Infinity,
  });
}

export function useQuarterFinalizationStatus(quarterId: string | null) {
  return useQuery({
    queryKey: ["quarter-finalization", quarterId],
    queryFn: () =>
      tauriInvoke<QuarterFinalizationStatus>("get_quarter_finalization_status", {
        quarterId,
      }),
    enabled: !!quarterId,
    staleTime: 10000,
  });
}

export function useUpsertQuarterOverride() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: {
      quarter_id: string;
      rule_key: string;
      incident_id: string;
      reason: string;
      approved_by?: string;
    }) =>
      tauriInvoke<QuarterOverride>("upsert_quarter_override", {
        req,
      }),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: ["quarter-finalization", vars.quarter_id] });
      qc.invalidateQueries({ queryKey: ["quarter-readiness", vars.quarter_id] });
    },
  });
}

export function useDeleteQuarterOverride() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (vars: { id: string; quarterId: string }) =>
      tauriInvoke<void>("delete_quarter_override", { id: vars.id }),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: ["quarter-finalization", vars.quarterId] });
    },
  });
}

export function useFinalizeQuarter() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: { quarter_id: string; finalized_by?: string; notes?: string }) =>
      tauriInvoke<{ finalization: unknown; snapshot: unknown }>("finalize_quarter", { req }),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: ["quarter-finalization", vars.quarter_id] });
      qc.invalidateQueries({ queryKey: ["quarter-readiness", vars.quarter_id] });
    },
  });
}

export function useUnfinalizeQuarter() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (quarterId: string) =>
      tauriInvoke<void>("unfinalize_quarter", { quarterId }),
    onSuccess: (_data, quarterId) => {
      qc.invalidateQueries({ queryKey: ["quarter-finalization", quarterId] });
    },
  });
}
