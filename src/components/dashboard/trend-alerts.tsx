import { TrendingUp, AlertTriangle } from "lucide-react";
import { useServiceTrends } from "@/hooks/use-ai";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

export function TrendAlerts() {
  const { data: trends } = useServiceTrends();

  if (!trends || trends.length === 0) return null;

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2 text-base">
          <TrendingUp className="h-4 w-4 text-orange-500" />
          Service Trend Alerts
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          {trends.map((trend, i) => (
            <div
              key={`${trend.service_id}-${trend.trend_type}-${i}`}
              className="flex items-start gap-3 rounded-md border p-3"
            >
              <AlertTriangle
                className={`mt-0.5 h-4 w-4 shrink-0 ${
                  trend.trend_type === "degrading"
                    ? "text-red-500"
                    : "text-yellow-500"
                }`}
              />
              <div className="flex-1 space-y-1">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium">
                    {trend.service_name}
                  </span>
                  <Badge
                    variant={
                      trend.trend_type === "degrading"
                        ? "destructive"
                        : "secondary"
                    }
                    className="text-[10px]"
                  >
                    {trend.trend_type === "degrading"
                      ? "Degrading"
                      : "High Volume"}
                  </Badge>
                </div>
                <p className="text-xs text-muted-foreground">
                  {trend.message}
                </p>
              </div>
              <div className="text-right text-sm">
                <div className="font-medium">{trend.incident_count_current}</div>
                <div className="text-xs text-muted-foreground">
                  vs {trend.incident_count_previous}
                </div>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
