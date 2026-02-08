import { useCallback } from "react";
import { BrowserRouter, Routes, Route, Navigate, useNavigate } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AppLayout } from "@/components/layout/app-layout";
import { QuickAddButton } from "@/components/layout/quick-add-button";
import { ErrorBoundary } from "@/components/error-boundary";
import { Toaster } from "@/components/ui/toaster";
import { DashboardView } from "@/views/dashboard-view";
import { IncidentsView } from "@/views/incidents-view";
import { IncidentDetailView } from "@/views/incident-detail-view";
import { ReportsView } from "@/views/reports-view";
import { SettingsView } from "@/views/settings-view";
import { useKeyboardShortcuts } from "@/hooks/use-keyboard";

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

  const handleQuickAdd = useCallback(() => {
    navigate("/incidents/new");
  }, [navigate]);

  useKeyboardShortcuts({
    onQuickAdd: handleQuickAdd,
  });

  return (
    <AppLayout>
      <Routes>
        <Route path="/" element={<Navigate to="/dashboard" replace />} />
        <Route path="/dashboard" element={<DashboardView />} />
        <Route path="/incidents" element={<IncidentsView />} />
        <Route path="/incidents/new" element={<IncidentDetailView />} />
        <Route path="/incidents/:id" element={<IncidentDetailView />} />
        <Route path="/reports" element={<ReportsView />} />
        <Route path="/settings" element={<SettingsView />} />
      </Routes>
      <QuickAddButton onQuickAdd={handleQuickAdd} />
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
