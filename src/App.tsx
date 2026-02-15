import { Suspense, lazy, useCallback, useState, type ReactNode } from "react";
import { BrowserRouter, Routes, Route, Navigate, useNavigate } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AppLayout } from "@/components/layout/app-layout";
import { QuickAddButton } from "@/components/layout/quick-add-button";
import { ErrorBoundary } from "@/components/error-boundary";
import { Toaster } from "@/components/ui/toaster";
import { useKeyboardShortcuts } from "@/hooks/use-keyboard";
import { useOnboardingComplete } from "@/hooks/use-onboarding";

const DashboardView = lazy(() =>
  import("@/views/dashboard-view").then((module) => ({
    default: module.DashboardView,
  }))
);
const IncidentsView = lazy(() =>
  import("@/views/incidents-view").then((module) => ({
    default: module.IncidentsView,
  }))
);
const IncidentDetailView = lazy(() =>
  import("@/views/incident-detail-view").then((module) => ({
    default: module.IncidentDetailView,
  }))
);
const ReportsView = lazy(() =>
  import("@/views/reports-view").then((module) => ({
    default: module.ReportsView,
  }))
);
const QuarterReviewView = lazy(() =>
  import("@/views/quarter-review-view").then((module) => ({
    default: module.QuarterReviewView,
  }))
);
const SettingsView = lazy(() =>
  import("@/views/settings-view").then((module) => ({
    default: module.SettingsView,
  }))
);
const TrashView = lazy(() =>
  import("@/views/trash-view").then((module) => ({
    default: module.TrashView,
  }))
);
const ActionItemsView = lazy(() =>
  import("@/views/action-items-view").then((module) => ({
    default: module.ActionItemsView,
  }))
);
const ServiceDetailView = lazy(() =>
  import("@/views/service-detail-view").then((module) => ({
    default: module.ServiceDetailView,
  }))
);
const LearningsView = lazy(() =>
  import("@/views/learnings-view").then((module) => ({
    default: module.LearningsView,
  }))
);
const ShiftHandoffView = lazy(() =>
  import("@/views/shift-handoff-view").then((module) => ({
    default: module.ShiftHandoffView,
  }))
);
const CommandPalette = lazy(() =>
  import("@/components/layout/command-palette").then((module) => ({
    default: module.CommandPalette,
  }))
);
const OnboardingWizard = lazy(() =>
  import("@/components/onboarding/onboarding-wizard").then((module) => ({
    default: module.OnboardingWizard,
  }))
);

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
        <Route path="/dashboard" element={<RouteSuspense><DashboardView /></RouteSuspense>} />
        <Route path="/incidents" element={<RouteSuspense><IncidentsView /></RouteSuspense>} />
        <Route path="/action-items" element={<RouteSuspense><ActionItemsView /></RouteSuspense>} />
        <Route path="/incidents/new" element={<RouteSuspense><IncidentDetailView /></RouteSuspense>} />
        <Route path="/incidents/:id" element={<RouteSuspense><IncidentDetailView /></RouteSuspense>} />
        <Route path="/services/:id" element={<RouteSuspense><ServiceDetailView /></RouteSuspense>} />
        <Route path="/learnings" element={<RouteSuspense><LearningsView /></RouteSuspense>} />
        <Route path="/handoff" element={<RouteSuspense><ShiftHandoffView /></RouteSuspense>} />
        <Route path="/quarter-review" element={<RouteSuspense><QuarterReviewView /></RouteSuspense>} />
        <Route path="/reports" element={<RouteSuspense><ReportsView /></RouteSuspense>} />
        <Route path="/settings" element={<RouteSuspense><SettingsView /></RouteSuspense>} />
        <Route path="/trash" element={<RouteSuspense><TrashView /></RouteSuspense>} />
      </Routes>
      <QuickAddButton onQuickAdd={handleQuickAdd} />
      {paletteOpen ? (
        <Suspense fallback={null}>
          <CommandPalette open={paletteOpen} onOpenChange={setPaletteOpen} />
        </Suspense>
      ) : null}
      {showOnboarding && (
        <Suspense fallback={null}>
          <OnboardingWizard
            open={true}
            onComplete={() => setOnboardingDismissed(true)}
          />
        </Suspense>
      )}
    </AppLayout>
  );
}

function RouteSuspense({ children }: { children: ReactNode }) {
  return (
    <Suspense fallback={<RouteFallback />}>
      {children}
    </Suspense>
  );
}

function RouteFallback() {
  return (
    <div className="p-6 text-sm text-muted-foreground">
      Loading view...
    </div>
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
