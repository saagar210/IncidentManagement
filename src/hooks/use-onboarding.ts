import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";

export function useOnboardingComplete() {
  return useQuery({
    queryKey: ["setting", "onboarding_complete"],
    queryFn: async () => {
      const val = await tauriInvoke<string | null>("get_setting", {
        key: "onboarding_complete",
      });
      return val === "true";
    },
    staleTime: Infinity,
  });
}

export function useCompleteOnboarding() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () =>
      tauriInvoke<void>("set_setting", {
        key: "onboarding_complete",
        value: "true",
      }),
    onSuccess: () =>
      qc.invalidateQueries({ queryKey: ["setting", "onboarding_complete"] }),
  });
}
