import { useServiceReliability } from "@/hooks/use-analytics";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

interface ServiceReliabilityScorecardProps {
  startDate: string;
  endDate: string;
}

function complianceBadge(pct: number) {
  if (pct >= 95) {
    return (
      <Badge variant="outline" className="bg-green-500/10 text-green-600 border-green-500/20">
        {pct.toFixed(1)}%
      </Badge>
    );
  }
  if (pct >= 80) {
    return (
      <Badge variant="outline" className="bg-yellow-500/10 text-yellow-600 border-yellow-500/20">
        {pct.toFixed(1)}%
      </Badge>
    );
  }
  return (
    <Badge variant="outline" className="bg-red-500/10 text-red-500 border-red-500/20">
      {pct.toFixed(1)}%
    </Badge>
  );
}

export function ServiceReliabilityScorecard({
  startDate,
  endDate,
}: ServiceReliabilityScorecardProps) {
  const { data: scores, isLoading } = useServiceReliability(startDate, endDate);

  if (isLoading || !scores) return null;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Service Reliability</CardTitle>
      </CardHeader>
      <CardContent>
        {scores.length === 0 ? (
          <p className="flex h-32 items-center justify-center text-sm text-muted-foreground">
            No incident data for this period
          </p>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Service</TableHead>
                <TableHead className="text-right">Incidents</TableHead>
                <TableHead className="text-right">Avg MTTR</TableHead>
                <TableHead className="text-right">SLA Compliance</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {scores.map((s) => (
                <TableRow key={s.service_id}>
                  <TableCell className="font-medium">{s.service_name}</TableCell>
                  <TableCell className="text-right">{s.incident_count}</TableCell>
                  <TableCell className="text-right text-sm">
                    {s.mttr_formatted}
                  </TableCell>
                  <TableCell className="text-right">
                    {complianceBadge(s.sla_compliance_pct)}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </CardContent>
    </Card>
  );
}
