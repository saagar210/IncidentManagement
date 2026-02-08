import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";

export interface ShiftHandoff {
  id: string;
  shift_end_time: string | null;
  content: string;
  created_by: string;
  created_at: string;
}

export interface CreateShiftHandoffRequest {
  shift_end_time?: string;
  content: string;
  created_by?: string;
}

export function useShiftHandoffs(limit?: number) {
  return useQuery({
    queryKey: ["shift-handoffs", limit],
    queryFn: () =>
      tauriInvoke<ShiftHandoff[]>("list_shift_handoffs", {
        limit: limit ?? 20,
      }),
  });
}

export function useCreateShiftHandoff() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: CreateShiftHandoffRequest) =>
      tauriInvoke<ShiftHandoff>("create_shift_handoff", { req }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["shift-handoffs"] });
    },
  });
}

export function useDeleteShiftHandoff() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_shift_handoff", { id }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["shift-handoffs"] });
    },
  });
}
