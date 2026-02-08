import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  Cell,
} from "recharts";
import { useEscalationFunnel } from "@/hooks/use-analytics";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { CHART_COLORS } from "@/lib/constants";

interface EscalationFunnelProps {
  startDate: string;
  endDate: string;
}

export function EscalationFunnel({ startDate, endDate }: EscalationFunnelProps) {
  const { data: entries, isLoading } = useEscalationFunnel(startDate, endDate);

  if (isLoading || !entries) return null;

  const chartData = entries.map((e) => ({
    name: e.severity,
    count: e.count,
    pct: e.percentage.toFixed(1),
    color:
      CHART_COLORS.severity[e.severity as keyof typeof CHART_COLORS.severity] ??
      "hsl(240, 5%, 65%)",
  }));

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Escalation Funnel</CardTitle>
      </CardHeader>
      <CardContent>
        {chartData.length === 0 ? (
          <p className="flex h-48 items-center justify-center text-sm text-muted-foreground">
            No incidents in this period
          </p>
        ) : (
          <ResponsiveContainer width="100%" height={200}>
            <BarChart data={chartData} layout="vertical">
              <XAxis type="number" allowDecimals={false} tick={{ fontSize: 12 }} />
              <YAxis
                type="category"
                dataKey="name"
                width={70}
                tick={{ fontSize: 12 }}
              />
              <Tooltip
                formatter={(value) => {
                  return [`${value}`, "Count"];
                }}
                labelFormatter={(label) => {
                  const entry = chartData.find((e) => e.name === label);
                  return entry ? `${label} (${entry.pct}%)` : label;
                }}
              />
              <Bar dataKey="count" radius={[0, 4, 4, 0]}>
                {chartData.map((entry, index) => (
                  <Cell key={index} fill={entry.color} />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
