import { useCallback, useState } from "react";
import { BrowserRouter, Routes, Route, Navigate, useNavigate } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AppLayout } from "@/components/layout/app-layout";
import { QuickAddButton } from "@/components/layout/quick-add-button";
import { CommandPalette } from "@/components/layout/command-palette";
import { OnboardingWizard } from "@/components/onboarding/onboarding-wizard";
import { ErrorBoundary } from "@/components/error-boundary";
import { Toaster } from "@/components/ui/toaster";
import { DashboardView } from "@/views/dashboard-view";
import { IncidentsView } from "@/views/incidents-view";
import { IncidentDetailView } from "@/views/incident-detail-view";
import { ReportsView } from "@/views/reports-view";
import { SettingsView } from "@/views/settings-view";
import { TrashView } from "@/views/trash-view";
import { ActionItemsView } from "@/views/action-items-view";
import { ServiceDetailView } from "@/views/service-detail-view";
import { LearningsView } from "@/views/learnings-view";
import { ShiftHandoffView } from "@/views/shift-handoff-view";
import { useKeyboardShortcuts } from "@/hooks/use-keyboard";
import { useOnboardingComplete } from "@/hooks/use-onboarding";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
    },
    mutations: {
      onError: (error: unknown) => {
        console.error("Mutation error:", error);
      },
    },
  },
});

function AppRoutes() {
  const navigate = useNavigate();
  const [paletteOpen, setPaletteOpen] = useState(false);
  const { data: onboardingDone, isLoading: onboardingLoading } = useOnboardingComplete();
  const [onboardingDismissed, setOnboardingDismissed] = useState(false);

  const handleQuickAdd = useCallback(() => {
    navigate("/incidents/new");
  }, [navigate]);

  const handleSearch = useCallback(() => {
    setPaletteOpen(true);
  }, []);

  useKeyboardShortcuts({
    onQuickAdd: handleQuickAdd,
    onSearch: handleSearch,
  });

  const showOnboarding = !onboardingLoading && !onboardingDone && !onboardingDismissed;

  return (
    <AppLayout>
      <Routes>
        <Route path="/" element={<Navigate to="/dashboard" replace />} />
        <Route path="/dashboard" element={<DashboardView />} />
        <Route path="/incidents" element={<IncidentsView />} />
        <Route path="/action-items" element={<ActionItemsView />} />
        <Route path="/incidents/new" element={<IncidentDetailView />} />
        <Route path="/incidents/:id" element={<IncidentDetailView />} />
        <Route path="/services/:id" element={<ServiceDetailView />} />
        <Route path="/learnings" element={<LearningsView />} />
        <Route path="/handoff" element={<ShiftHandoffView />} />
        <Route path="/reports" element={<ReportsView />} />
        <Route path="/settings" element={<SettingsView />} />
        <Route path="/trash" element={<TrashView />} />
      </Routes>
      <QuickAddButton onQuickAdd={handleQuickAdd} />
      <CommandPalette open={paletteOpen} onOpenChange={setPaletteOpen} />
      {showOnboarding && (
        <OnboardingWizard
          open={true}
          onComplete={() => setOnboardingDismissed(true)}
        />
      )}
    </AppLayout>
  );
}

export default function App() {
  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AppRoutes />
          <Toaster />
        </BrowserRouter>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}
