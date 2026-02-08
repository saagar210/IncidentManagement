import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type {
  SavedFilter,
  CreateSavedFilterRequest,
  UpdateSavedFilterRequest,
} from "@/types/analytics";

export function useSavedFilters() {
  return useQuery({
    queryKey: ["saved-filters"],
    queryFn: () => tauriInvoke<SavedFilter[]>("list_saved_filters"),
  });
}

export function useCreateSavedFilter() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (req: CreateSavedFilterRequest) =>
      tauriInvoke<SavedFilter>("create_saved_filter", { req }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["saved-filters"] });
    },
  });
}

export function useUpdateSavedFilter() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, req }: { id: string; req: UpdateSavedFilterRequest }) =>
      tauriInvoke<SavedFilter>("update_saved_filter", { id, req }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["saved-filters"] });
    },
  });
}

export function useDeleteSavedFilter() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_saved_filter", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["saved-filters"] });
    },
  });
}
