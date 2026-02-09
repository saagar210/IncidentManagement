import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type {
  ContributingFactor,
  CreateContributingFactorRequest,
  Postmortem,
  PostmortemTemplate,
  CreatePostmortemRequest,
  UpdatePostmortemRequest,
  PostmortemReadiness,
} from "@/types/postmortem";

// --- Contributing Factors ---

export function useContributingFactors(incidentId: string | undefined) {
  return useQuery({
    queryKey: ["contributing-factors", incidentId],
    queryFn: () =>
      tauriInvoke<ContributingFactor[]>("list_contributing_factors", {
        incidentId,
      }),
    enabled: !!incidentId,
  });
}

export function useCreateContributingFactor() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (req: CreateContributingFactorRequest) =>
      tauriInvoke<ContributingFactor>("create_contributing_factor", { req }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({
        queryKey: ["contributing-factors", variables.incident_id],
      });
      queryClient.invalidateQueries({ queryKey: ["notification-summary"] });
    },
  });
}

export function useDeleteContributingFactor() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_contributing_factor", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["contributing-factors"] });
    },
  });
}

// --- Post-mortem Templates ---

export function usePostmortemTemplates() {
  return useQuery({
    queryKey: ["postmortem-templates"],
    queryFn: () =>
      tauriInvoke<PostmortemTemplate[]>("list_postmortem_templates"),
  });
}

// --- Postmortems ---

export function usePostmortemByIncident(incidentId: string | undefined) {
  return useQuery({
    queryKey: ["postmortem", incidentId],
    queryFn: () =>
      tauriInvoke<Postmortem | null>("get_postmortem_by_incident", {
        incidentId,
      }),
    enabled: !!incidentId,
  });
}

export function useCreatePostmortem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (req: CreatePostmortemRequest) =>
      tauriInvoke<Postmortem>("create_postmortem", { req }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({
        queryKey: ["postmortem", variables.incident_id],
      });
      queryClient.invalidateQueries({ queryKey: ["notification-summary"] });
    },
  });
}

export function useUpdatePostmortem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, req }: { id: string; req: UpdatePostmortemRequest }) =>
      tauriInvoke<Postmortem>("update_postmortem", { id, req }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["postmortem"] });
    },
  });
}

export function useDeletePostmortem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_postmortem", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["postmortem"] });
    },
  });
}

export function usePostmortemReadiness(incidentId: string | undefined) {
  return useQuery({
    queryKey: ["postmortem-readiness", incidentId],
    queryFn: () =>
      tauriInvoke<PostmortemReadiness>("get_postmortem_readiness", { incidentId }),
    enabled: !!incidentId,
  });
}
