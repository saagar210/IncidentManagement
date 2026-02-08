import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type {
  Service,
  CreateServiceRequest,
  UpdateServiceRequest,
} from "@/types/incident";

export function useServices() {
  return useQuery({
    queryKey: ["services"],
    queryFn: () => tauriInvoke<Service[]>("list_services"),
  });
}

export function useActiveServices() {
  return useQuery({
    queryKey: ["services", "active"],
    queryFn: () => tauriInvoke<Service[]>("list_active_services"),
  });
}

export function useCreateService() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (service: CreateServiceRequest) =>
      tauriInvoke<Service>("create_service", { service }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["services"] });
    },
  });
}

export function useUpdateService() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, service }: { id: string; service: UpdateServiceRequest }) =>
      tauriInvoke<Service>("update_service", { id, service }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["services"] });
    },
  });
}

export function useDeleteService() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_service", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["services"] });
    },
  });
}
