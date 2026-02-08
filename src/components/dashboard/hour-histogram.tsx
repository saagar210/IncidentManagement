import { useMemo } from "react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { HourCount } from "@/types/metrics";

interface HourHistogramProps {
  data: HourCount[];
}

export function HourHistogram({ data }: HourHistogramProps) {
  const chartData = useMemo(() => {
    const countMap = new Map<number, number>();
    for (const d of data) {
      countMap.set(d.hour, d.count);
    }
    return Array.from({ length: 24 }, (_, i) => ({
      hour: i,
      label: `${i.toString().padStart(2, "0")}:00`,
      count: countMap.get(i) ?? 0,
    }));
  }, [data]);

  const hasData = data.length > 0;

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-base">Incidents by Time of Day</CardTitle>
      </CardHeader>
      <CardContent>
        {!hasData ? (
          <p className="flex h-48 items-center justify-center text-sm text-muted-foreground">
            No time-of-day data available
          </p>
        ) : (
          <ResponsiveContainer width="100%" height={200}>
            <BarChart data={chartData}>
              <XAxis
                dataKey="hour"
                tick={{ fontSize: 10 }}
                tickFormatter={(h: number) => (h % 3 === 0 ? `${h}h` : "")}
              />
              <YAxis allowDecimals={false} tick={{ fontSize: 10 }} width={30} />
              <Tooltip
                formatter={(value: number | undefined) => [value ?? 0, "Incidents"]}
                labelFormatter={(h: unknown) => {
                  const hour = Number(h);
                  return `${hour.toString().padStart(2, "0")}:00 - ${hour.toString().padStart(2, "0")}:59`;
                }}
              />
              <Bar
                dataKey="count"
                fill="hsl(217, 91%, 60%)"
                radius={[2, 2, 0, 0]}
              />
            </BarChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
