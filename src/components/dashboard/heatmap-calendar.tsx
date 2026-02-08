import { useMemo } from "react";
import {
  startOfWeek,
  addDays,
  format,
  parseISO,
  differenceInWeeks,
  isBefore,
  isAfter,
} from "date-fns";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type { DayCount } from "@/types/metrics";

interface HeatmapCalendarProps {
  data: DayCount[];
  startDate: string;
  endDate: string;
}

const INTENSITY_CLASSES = [
  "bg-muted",
  "bg-green-200 dark:bg-green-900",
  "bg-green-400 dark:bg-green-700",
  "bg-green-600 dark:bg-green-500",
  "bg-green-800 dark:bg-green-300",
] as const;

const DAY_LABELS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

function getIntensity(count: number, max: number): number {
  if (count === 0) return 0;
  if (max === 0) return 0;
  const ratio = count / max;
  if (ratio <= 0.25) return 1;
  if (ratio <= 0.5) return 2;
  if (ratio <= 0.75) return 3;
  return 4;
}

export function HeatmapCalendar({
  data,
  startDate,
  endDate,
}: HeatmapCalendarProps) {
  const { grid, weeks, maxCount, monthLabels } = useMemo(() => {
    const countMap = new Map<string, number>();
    let maxCount = 0;
    for (const d of data) {
      countMap.set(d.day, d.count);
      if (d.count > maxCount) maxCount = d.count;
    }

    const start = parseISO(startDate);
    const end = parseISO(endDate);
    const weekStart = startOfWeek(start, { weekStartsOn: 0 });
    const totalWeeks = differenceInWeeks(end, weekStart) + 2;
    const weeks = Math.min(totalWeeks, 53);

    const grid: Array<{
      date: Date;
      dateStr: string;
      count: number;
      inRange: boolean;
    }> = [];

    const monthLabels: Array<{ label: string; weekIdx: number }> = [];
    let lastMonth = -1;

    for (let w = 0; w < weeks; w++) {
      for (let d = 0; d < 7; d++) {
        const date = addDays(weekStart, w * 7 + d);
        const dateStr = format(date, "yyyy-MM-dd");
        const inRange = !isBefore(date, start) && !isAfter(date, end);
        grid.push({
          date,
          dateStr,
          count: countMap.get(dateStr) ?? 0,
          inRange,
        });

        if (d === 0) {
          const month = date.getMonth();
          if (month !== lastMonth) {
            monthLabels.push({
              label: format(date, "MMM"),
              weekIdx: w,
            });
            lastMonth = month;
          }
        }
      }
    }

    return { grid, weeks, maxCount, monthLabels };
  }, [data, startDate, endDate]);

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-base">Incident Heatmap</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="overflow-x-auto">
          {/* Month labels */}
          <div className="flex pl-8 mb-1">
            {monthLabels.map((m, i) => (
              <div
                key={i}
                className="text-[10px] text-muted-foreground"
                style={{
                  position: "relative",
                  left: `${m.weekIdx * 14}px`,
                  marginRight: i < monthLabels.length - 1
                    ? `${((monthLabels[i + 1]?.weekIdx ?? weeks) - m.weekIdx) * 14 - 24}px`
                    : 0,
                }}
              >
                {m.label}
              </div>
            ))}
          </div>
          <div className="flex gap-0.5">
            {/* Day labels */}
            <div className="flex flex-col gap-0.5 pr-1">
              {DAY_LABELS.map((label, i) => (
                <div
                  key={i}
                  className="h-3 w-6 text-[9px] text-muted-foreground leading-3"
                >
                  {i % 2 === 1 ? label : ""}
                </div>
              ))}
            </div>
            {/* Grid */}
            {Array.from({ length: weeks }).map((_, weekIdx) => (
              <div key={weekIdx} className="flex flex-col gap-0.5">
                {Array.from({ length: 7 }).map((_, dayIdx) => {
                  const cell = grid[weekIdx * 7 + dayIdx];
                  if (!cell) return <div key={dayIdx} className="h-3 w-3" />;
                  const intensity = cell.inRange
                    ? getIntensity(cell.count, maxCount)
                    : -1;
                  return (
                    <div
                      key={dayIdx}
                      className={cn(
                        "h-3 w-3 rounded-[2px]",
                        intensity === -1
                          ? "bg-transparent"
                          : INTENSITY_CLASSES[intensity]
                      )}
                      title={
                        cell.inRange
                          ? `${format(cell.date, "MMM d, yyyy")}: ${cell.count} incident${cell.count !== 1 ? "s" : ""}`
                          : ""
                      }
                    />
                  );
                })}
              </div>
            ))}
          </div>
          {/* Legend */}
          <div className="mt-2 flex items-center gap-1 text-[10px] text-muted-foreground">
            <span>Less</span>
            {INTENSITY_CLASSES.map((cls, i) => (
              <div
                key={i}
                className={cn("h-3 w-3 rounded-[2px]", cls)}
              />
            ))}
            <span>More</span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
