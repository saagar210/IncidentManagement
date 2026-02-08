import { useNavigate } from "react-router-dom";
import { LinkIcon } from "lucide-react";
import { useSimilarIncidents } from "@/hooks/use-ai";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { SEVERITY_COLORS } from "@/lib/constants";
import type { SeverityLevel } from "@/lib/constants";

interface SimilarIncidentsPanelProps {
  query: string;
  excludeId?: string;
}

export function SimilarIncidentsPanel({
  query,
  excludeId,
}: SimilarIncidentsPanelProps) {
  const navigate = useNavigate();
  const { data: similar } = useSimilarIncidents(query, excludeId);

  if (!similar || similar.length === 0) return null;

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2 text-base">
          <LinkIcon className="h-4 w-4 text-blue-500" />
          Similar Incidents
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          {similar.map((inc) => (
            <div
              key={inc.id}
              className="flex cursor-pointer items-center justify-between rounded-md border p-2 hover:bg-accent"
              onClick={() => navigate(`/incidents/${inc.id}`)}
            >
              <div className="flex-1 truncate">
                <p className="text-sm font-medium truncate">{inc.title}</p>
                <p className="text-xs text-muted-foreground">
                  {inc.service_name}
                </p>
              </div>
              <div className="flex items-center gap-2">
                <Badge
                  variant="outline"
                  className={
                    SEVERITY_COLORS[inc.severity as SeverityLevel] ?? ""
                  }
                >
                  {inc.severity}
                </Badge>
                <Badge variant="outline">{inc.status}</Badge>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
