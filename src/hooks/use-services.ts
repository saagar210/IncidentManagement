import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type {
  Service,
  CreateServiceRequest,
  UpdateServiceRequest,
  ServiceDependency,
  CreateServiceDependencyRequest,
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

export function useService(id: string | undefined) {
  return useQuery({
    queryKey: ["service", id],
    queryFn: () => tauriInvoke<Service>("get_service", { id }),
    enabled: !!id,
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
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["services"] });
      queryClient.invalidateQueries({ queryKey: ["service", variables.id] });
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

// Service dependency hooks

export function useServiceDependencies(serviceId: string | undefined) {
  return useQuery({
    queryKey: ["service-dependencies", serviceId],
    queryFn: () =>
      tauriInvoke<ServiceDependency[]>("list_service_dependencies", {
        serviceId,
      }),
    enabled: !!serviceId,
  });
}

export function useServiceDependents(serviceId: string | undefined) {
  return useQuery({
    queryKey: ["service-dependents", serviceId],
    queryFn: () =>
      tauriInvoke<ServiceDependency[]>("list_service_dependents", {
        serviceId,
      }),
    enabled: !!serviceId,
  });
}

export function useAddServiceDependency() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (req: CreateServiceDependencyRequest) =>
      tauriInvoke<ServiceDependency>("add_service_dependency", { req }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({
        queryKey: ["service-dependencies", variables.service_id],
      });
      queryClient.invalidateQueries({
        queryKey: ["service-dependents"],
      });
    },
  });
}

export function useRemoveServiceDependency() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("remove_service_dependency", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["service-dependencies"] });
      queryClient.invalidateQueries({ queryKey: ["service-dependents"] });
    },
  });
}
