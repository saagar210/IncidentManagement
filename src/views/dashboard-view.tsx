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
import { TrendingUp, TrendingDown, Minus } from "lucide-react";
import { useDashboardData } from "@/hooks/use-metrics";
import { useQuarters } from "@/hooks/use-quarters";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Select } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { CHART_COLORS } from "@/lib/constants";
import { cn } from "@/lib/utils";
import type { MetricResult, CategoryCount } from "@/types/metrics";

function TrendIcon({ trend }: { trend: MetricResult["trend"] }) {
  switch (trend) {
    case "Up":
      return <TrendingUp className="h-4 w-4" />;
    case "Down":
      return <TrendingDown className="h-4 w-4" />;
    case "Flat":
      return <Minus className="h-4 w-4" />;
    default:
      return null;
  }
}

function trendColor(trend: MetricResult["trend"], invertGood = false): string {
  // For MTTR/MTTA, Down is good. For recurrence, Down is good. For tickets, context-dependent.
  const isGood = invertGood
    ? trend === "Up"
    : trend === "Down";
  if (trend === "Flat" || trend === "NoData") return "text-muted-foreground";
  return isGood ? "text-green-500" : "text-red-500";
}

interface MetricCardProps {
  label: string;
  metric: MetricResult;
  invertGood?: boolean;
}

function MetricCard({ label, metric, invertGood = false }: MetricCardProps) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium text-muted-foreground">
          {label}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold">{metric.formatted_value}</div>
        <div className={cn("mt-1 flex items-center gap-1 text-xs", trendColor(metric.trend, invertGood))}>
          <TrendIcon trend={metric.trend} />
          {metric.previous_value !== null && (
            <span>prev: {metric.previous_value.toFixed(1)}</span>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

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
  const { data: dashboard, isLoading: dashboardLoading } = useDashboardData(activeQuarterId);

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

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <MetricCard label="MTTR (Mean Time to Resolve)" metric={dashboard.mttr} />
        <MetricCard label="MTTA (Mean Time to Acknowledge)" metric={dashboard.mtta} />
        <MetricCard label="Recurrence Rate" metric={dashboard.recurrence_rate} />
        <MetricCard label="Avg Tickets / Incident" metric={dashboard.avg_tickets} invertGood />
      </div>

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
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
      </div>
    </div>
  );
}
