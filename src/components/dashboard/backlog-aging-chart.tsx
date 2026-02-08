import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  Cell,
} from "recharts";
import { useBacklogAging } from "@/hooks/use-analytics";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

const AGING_COLORS = [
  "hsl(142, 76%, 36%)", // green - 0-1 day
  "hsl(48, 96%, 53%)",  // yellow - 1-3 days
  "hsl(36, 100%, 50%)", // orange - 3-7 days
  "hsl(14, 100%, 57%)", // red-orange - 7-14 days
  "hsl(0, 84%, 60%)",   // red - 14+ days
];

export function BacklogAgingChart() {
  const { data: buckets, isLoading } = useBacklogAging();

  if (isLoading || !buckets) return null;

  const totalOpen = buckets.reduce((sum, b) => sum + b.count, 0);

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">
          Backlog Aging
          <span className="ml-2 text-sm font-normal text-muted-foreground">
            {totalOpen} open
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent>
        {totalOpen === 0 ? (
          <p className="flex h-48 items-center justify-center text-sm text-muted-foreground">
            No open incidents
          </p>
        ) : (
          <ResponsiveContainer width="100%" height={200}>
            <BarChart data={buckets}>
              <XAxis dataKey="label" tick={{ fontSize: 11 }} />
              <YAxis allowDecimals={false} tick={{ fontSize: 12 }} />
              <Tooltip />
              <Bar dataKey="count" radius={[4, 4, 0, 0]}>
                {buckets.map((_, index) => (
                  <Cell key={index} fill={AGING_COLORS[index] ?? AGING_COLORS[4]} />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
