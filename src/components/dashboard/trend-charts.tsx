import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { QuarterlyTrends } from "@/types/metrics";

interface TrendChartsProps {
  trends: QuarterlyTrends;
}

interface SparklineProps {
  title: string;
  data: Array<{ quarter: string; value: number }>;
  color: string;
  formatter?: (v: number) => string;
}

function Sparkline({ title, data, color, formatter }: SparklineProps) {
  if (data.length === 0) {
    return (
      <div className="space-y-1">
        <p className="text-xs font-medium text-muted-foreground">{title}</p>
        <p className="text-xs text-muted-foreground">No trend data</p>
      </div>
    );
  }

  return (
    <div className="space-y-1">
      <p className="text-xs font-medium text-muted-foreground">{title}</p>
      <ResponsiveContainer width="100%" height={80}>
        <LineChart data={data}>
          <XAxis dataKey="quarter" tick={{ fontSize: 9 }} />
          <YAxis hide />
          <Tooltip
            formatter={(value: number | undefined) => [
              formatter ? formatter(value ?? 0) : (value ?? 0).toFixed(1),
              title,
            ]}
          />
          <Line
            type="monotone"
            dataKey="value"
            stroke={color}
            strokeWidth={2}
            dot={{ r: 3, fill: color }}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}

function formatMinutes(mins: number): string {
  if (mins < 60) return `${mins.toFixed(0)} min`;
  const h = Math.floor(mins / 60);
  const m = Math.round(mins % 60);
  return m === 0 ? `${h}h` : `${h}h ${m}m`;
}

export function TrendCharts({ trends }: TrendChartsProps) {
  const quarters = trends.quarters;

  const mttrData = quarters.map((q, i) => ({
    quarter: q,
    value: trends.mttr[i] ?? 0,
  }));

  const mttaData = quarters.map((q, i) => ({
    quarter: q,
    value: trends.mtta[i] ?? 0,
  }));

  const countData = quarters.map((q, i) => ({
    quarter: q,
    value: trends.incident_count[i] ?? 0,
  }));

  const recurrenceData = quarters.map((q, i) => ({
    quarter: q,
    value: trends.recurrence_rate[i] ?? 0,
  }));

  const hasData = quarters.length > 0;

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-base">Quarterly Trends</CardTitle>
      </CardHeader>
      <CardContent>
        {!hasData ? (
          <p className="flex h-48 items-center justify-center text-sm text-muted-foreground">
            Not enough quarterly data for trends
          </p>
        ) : (
          <div className="grid grid-cols-2 gap-4">
            <Sparkline
              title="MTTR"
              data={mttrData}
              color="hsl(217, 91%, 60%)"
              formatter={formatMinutes}
            />
            <Sparkline
              title="MTTA"
              data={mttaData}
              color="hsl(142, 76%, 36%)"
              formatter={formatMinutes}
            />
            <Sparkline
              title="Incident Count"
              data={countData}
              color="hsl(0, 84%, 60%)"
              formatter={(v) => v.toFixed(0)}
            />
            <Sparkline
              title="Recurrence Rate"
              data={recurrenceData}
              color="hsl(38, 92%, 50%)"
              formatter={(v) => `${v.toFixed(1)}%`}
            />
          </div>
        )}
      </CardContent>
    </Card>
  );
}
