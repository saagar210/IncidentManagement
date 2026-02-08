import { useState } from "react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  Cell,
} from "recharts";
import { useDashboardData } from "@/hooks/use-metrics";
import { useQuarters } from "@/hooks/use-quarters";
import {
  useIncidentHeatmap,
  useIncidentByHour,
  useDashboardConfig,
  useUpdateDashboardConfig,
} from "@/hooks/use-dashboard";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Select } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { HeatmapCalendar } from "@/components/dashboard/heatmap-calendar";
import { HourHistogram } from "@/components/dashboard/hour-histogram";
import { TrendCharts } from "@/components/dashboard/trend-charts";
import { MetricCardConfig } from "@/components/dashboard/metric-card-config";
import { BacklogAgingChart } from "@/components/dashboard/backlog-aging-chart";
import { EscalationFunnel } from "@/components/dashboard/escalation-funnel";
import { ServiceReliabilityScorecard } from "@/components/dashboard/service-reliability-scorecard";
import { PeriodComparisonCard } from "@/components/dashboard/period-comparison-card";
import { TrendAlerts } from "@/components/dashboard/trend-alerts";
import { CHART_COLORS } from "@/lib/constants";
import type { CategoryCount } from "@/types/metrics";

function DashboardSkeleton() {
  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-9 w-48" />
      </div>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <Card key={i}>
            <CardHeader className="pb-2">
              <Skeleton className="h-4 w-24" />
            </CardHeader>
            <CardContent>
              <Skeleton className="h-8 w-20" />
              <Skeleton className="mt-2 h-3 w-16" />
            </CardContent>
          </Card>
        ))}
      </div>
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <Skeleton className="h-5 w-40" />
          </CardHeader>
          <CardContent>
            <Skeleton className="h-64 w-full" />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <Skeleton className="h-5 w-40" />
          </CardHeader>
          <CardContent>
            <Skeleton className="h-64 w-full" />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

export function DashboardView() {
  const { data: quarters, isLoading: quartersLoading } = useQuarters();
  const [selectedQuarterId, setSelectedQuarterId] = useState<string | null>(null);

  const activeQuarterId = selectedQuarterId ?? quarters?.[0]?.id ?? null;
  const selectedQuarter = quarters?.find((q) => q.id === activeQuarterId);

  const { data: dashboard, isLoading: dashboardLoading } = useDashboardData(activeQuarterId);
  const { data: cardConfig } = useDashboardConfig();
  const updateConfig = useUpdateDashboardConfig();

  // Derive date range for heatmap/histogram
  const startDate = selectedQuarter?.start_date?.split("T")[0] ?? "";
  const endDate = selectedQuarter?.end_date?.split("T")[0] ?? "";

  const { data: heatmapData } = useIncidentHeatmap(startDate, endDate);
  const { data: hourData } = useIncidentByHour(
    startDate || null,
    endDate || null
  );

  const config = cardConfig ?? {
    mttr: true,
    mtta: true,
    recurrence_rate: true,
    avg_tickets: true,
    by_severity: true,
    by_service: true,
    heatmap: true,
    hour_histogram: true,
    trends: true,
    timeline: false,
  };

  if (quartersLoading || dashboardLoading) {
    return <DashboardSkeleton />;
  }

  if (!dashboard) {
    return (
      <div className="flex h-full items-center justify-center p-6">
        <p className="text-muted-foreground">No dashboard data available. Create some incidents first.</p>
      </div>
    );
  }

  const severityData: Array<{ name: string; count: number; color: string }> =
    dashboard.by_severity.map((item: CategoryCount) => ({
      name: item.category,
      count: item.count,
      color:
        CHART_COLORS.severity[
          item.category as keyof typeof CHART_COLORS.severity
        ] ?? "hsl(240, 5%, 65%)",
    }));

  const serviceData: Array<{ name: string; count: number }> =
    dashboard.by_service.map((item: CategoryCount) => ({
      name: item.category,
      count: item.count,
    }));

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold">Dashboard</h1>
          <p className="text-sm text-muted-foreground">
            {dashboard.period_label} &middot; {dashboard.total_incidents} incidents
          </p>
        </div>
        <div className="flex items-center gap-2">
          <MetricCardConfig
            config={config}
            onUpdate={(c) => updateConfig.mutate(c)}
          />
          <Select
            value={activeQuarterId ?? ""}
            onChange={(e) => setSelectedQuarterId(e.target.value || null)}
            className="w-48"
          >
            <option value="">All time</option>
            {quarters?.map((q) => (
              <option key={q.id} value={q.id}>
                {q.label}
              </option>
            ))}
          </Select>
        </div>
      </div>

      {/* Service Trend Alerts */}
      <TrendAlerts />

      {/* Metric Cards — Period Comparison */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {config.mttr && (
          <PeriodComparisonCard
            label="MTTR (Mean Time to Resolve)"
            metric={dashboard.mttr}
            description="Lower is better"
          />
        )}
        {config.mtta && (
          <PeriodComparisonCard
            label="MTTA (Mean Time to Acknowledge)"
            metric={dashboard.mtta}
            description="Lower is better"
          />
        )}
        {config.recurrence_rate && (
          <PeriodComparisonCard
            label="Recurrence Rate"
            metric={dashboard.recurrence_rate}
            description="Lower is better"
          />
        )}
        {config.avg_tickets && (
          <PeriodComparisonCard
            label="Avg Tickets / Incident"
            metric={dashboard.avg_tickets}
            invertGood
          />
        )}
      </div>

      {/* Charts Row 1 */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        {config.by_severity && (
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Incidents by Severity</CardTitle>
            </CardHeader>
            <CardContent>
              {severityData.length === 0 ? (
                <p className="flex h-64 items-center justify-center text-sm text-muted-foreground">
                  No severity data
                </p>
              ) : (
                <ResponsiveContainer width="100%" height={280}>
                  <BarChart data={severityData}>
                    <XAxis dataKey="name" tick={{ fontSize: 12 }} />
                    <YAxis allowDecimals={false} tick={{ fontSize: 12 }} />
                    <Tooltip />
                    <Bar dataKey="count" radius={[4, 4, 0, 0]}>
                      {severityData.map((entry, index) => (
                        <Cell key={index} fill={entry.color} />
                      ))}
                    </Bar>
                  </BarChart>
                </ResponsiveContainer>
              )}
            </CardContent>
          </Card>
        )}

        {config.by_service && (
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Incidents by Service</CardTitle>
            </CardHeader>
            <CardContent>
              {serviceData.length === 0 ? (
                <p className="flex h-64 items-center justify-center text-sm text-muted-foreground">
                  No service data
                </p>
              ) : (
                <ResponsiveContainer width="100%" height={280}>
                  <BarChart data={serviceData} layout="vertical">
                    <XAxis type="number" allowDecimals={false} tick={{ fontSize: 12 }} />
                    <YAxis
                      type="category"
                      dataKey="name"
                      width={120}
                      tick={{ fontSize: 12 }}
                    />
                    <Tooltip />
                    <Bar dataKey="count" fill="hsl(217, 91%, 60%)" radius={[0, 4, 4, 0]} />
                  </BarChart>
                </ResponsiveContainer>
              )}
            </CardContent>
          </Card>
        )}
      </div>

      {/* Charts Row 2 - Heatmap and Hour Histogram */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        {config.heatmap && heatmapData && startDate && endDate && (
          <HeatmapCalendar
            data={heatmapData}
            startDate={startDate}
            endDate={endDate}
          />
        )}
        {config.hour_histogram && hourData && (
          <HourHistogram data={hourData} />
        )}
      </div>

      {/* Charts Row 3 - Analytics */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <BacklogAgingChart />
        {startDate && endDate && (
          <EscalationFunnel startDate={startDate} endDate={endDate} />
        )}
      </div>

      {/* Service Reliability Scorecard */}
      {startDate && endDate && (
        <ServiceReliabilityScorecard startDate={startDate} endDate={endDate} />
      )}

      {/* Trend Charts */}
      {config.trends && dashboard.trends.quarters.length > 0 && (
        <TrendCharts trends={dashboard.trends} />
      )}
    </div>
  );
}
