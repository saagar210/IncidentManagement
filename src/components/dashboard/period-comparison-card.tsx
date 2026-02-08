import { TrendingUp, TrendingDown, Minus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type { MetricResult } from "@/types/metrics";

interface PeriodComparisonCardProps {
  label: string;
  metric: MetricResult;
  invertGood?: boolean;
  description?: string;
}

function trendColor(trend: MetricResult["trend"], invertGood = false): string {
  const isGood = invertGood ? trend === "Up" : trend === "Down";
  if (trend === "Flat" || trend === "NoData") return "text-muted-foreground";
  return isGood ? "text-green-500" : "text-red-500";
}

function trendBg(trend: MetricResult["trend"], invertGood = false): string {
  const isGood = invertGood ? trend === "Up" : trend === "Down";
  if (trend === "Flat" || trend === "NoData") return "bg-muted";
  return isGood ? "bg-green-500/10" : "bg-red-500/10";
}

export function PeriodComparisonCard({
  label,
  metric,
  invertGood = false,
  description,
}: PeriodComparisonCardProps) {
  const hasPrevious = metric.previous_value != null;
  const changeValue =
    hasPrevious && metric.previous_value !== 0
      ? (((metric.value - metric.previous_value!) / Math.abs(metric.previous_value!)) * 100)
      : null;

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium text-muted-foreground">
          {label}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-2">
        <div className="text-2xl font-bold">{metric.formatted_value}</div>

        {hasPrevious && (
          <div
            className={cn(
              "inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-xs font-medium",
              trendBg(metric.trend, invertGood),
              trendColor(metric.trend, invertGood)
            )}
          >
            {metric.trend === "Up" && <TrendingUp className="h-3 w-3" />}
            {metric.trend === "Down" && <TrendingDown className="h-3 w-3" />}
            {metric.trend === "Flat" && <Minus className="h-3 w-3" />}
            {changeValue !== null
              ? `${changeValue > 0 ? "+" : ""}${changeValue.toFixed(1)}%`
              : metric.trend}
            <span className="text-muted-foreground ml-1">vs prev</span>
          </div>
        )}

        {description && (
          <p className="text-xs text-muted-foreground">{description}</p>
        )}
      </CardContent>
    </Card>
  );
}
