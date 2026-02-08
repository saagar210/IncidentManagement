import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type {
  CustomFieldDefinition,
  CreateCustomFieldRequest,
  UpdateCustomFieldRequest,
  CustomFieldValue,
} from "@/types/custom-fields";

export function useCustomFields() {
  return useQuery({
    queryKey: ["custom-fields"],
    queryFn: () => tauriInvoke<CustomFieldDefinition[]>("list_custom_fields"),
    staleTime: Infinity,
  });
}

export function useCreateCustomField() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (field: CreateCustomFieldRequest) =>
      tauriInvoke<CustomFieldDefinition>("create_custom_field", { field }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["custom-fields"] }),
  });
}

export function useUpdateCustomField() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, field }: { id: string; field: UpdateCustomFieldRequest }) =>
      tauriInvoke<CustomFieldDefinition>("update_custom_field", { id, field }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["custom-fields"] }),
  });
}

export function useDeleteCustomField() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_custom_field", { id }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["custom-fields"] }),
  });
}

export function useIncidentCustomFields(incidentId: string | undefined) {
  return useQuery({
    queryKey: ["incident-custom-fields", incidentId],
    queryFn: () =>
      tauriInvoke<CustomFieldValue[]>("get_incident_custom_fields", {
        incidentId: incidentId as string,
      }),
    enabled: !!incidentId,
  });
}

export function useSetIncidentCustomFields() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      incidentId,
      values,
    }: {
      incidentId: string;
      values: CustomFieldValue[];
    }) =>
      tauriInvoke<CustomFieldValue[]>("set_incident_custom_fields", {
        incidentId,
        values,
      }),
    onSuccess: (_, vars) =>
      qc.invalidateQueries({
        queryKey: ["incident-custom-fields", vars.incidentId],
      }),
  });
}
