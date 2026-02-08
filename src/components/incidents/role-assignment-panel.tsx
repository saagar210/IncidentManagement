import { useState } from "react";
import { Plus, X, UserCircle, Shield } from "lucide-react";
import {
  useIncidentRoles,
  useAssignRole,
  useUnassignRole,
} from "@/hooks/use-roles";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { toast } from "@/components/ui/use-toast";
import { INCIDENT_ROLES, type IncidentRoleType } from "@/lib/constants";

interface RoleAssignmentPanelProps {
  incidentId: string;
}

const ROLE_ICONS: Record<string, string> = {
  "Incident Commander": "IC",
  "Communications Lead": "CL",
  "Technical Lead": "TL",
  Scribe: "SC",
  SME: "SME",
};

export function RoleAssignmentPanel({ incidentId }: RoleAssignmentPanelProps) {
  const { data: roles, isLoading } = useIncidentRoles(incidentId);
  const assignRole = useAssignRole();
  const unassignRole = useUnassignRole();

  const [adding, setAdding] = useState(false);
  const [newRole, setNewRole] = useState<IncidentRoleType>(INCIDENT_ROLES[0]);
  const [newAssignee, setNewAssignee] = useState("");

  const handleAssign = async () => {
    if (!newAssignee.trim()) return;
    try {
      await assignRole.mutateAsync({
        incident_id: incidentId,
        role: newRole,
        assignee: newAssignee.trim(),
      });
      setAdding(false);
      setNewAssignee("");
      toast({ title: "Role assigned" });
    } catch (err) {
      toast({
        title: "Failed to assign role",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleUnassign = async (roleId: string) => {
    try {
      await unassignRole.mutateAsync(roleId);
      toast({ title: "Role unassigned" });
    } catch (err) {
      toast({
        title: "Failed to unassign",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  if (isLoading) {
    return <p className="text-sm text-muted-foreground">Loading roles...</p>;
  }

  // Group by role
  const grouped = (roles ?? []).reduce<Record<string, typeof roles>>((acc, r) => {
    if (!acc[r.role]) acc[r.role] = [];
    acc[r.role]!.push(r);
    return acc;
  }, {});

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="flex items-center gap-1.5 text-sm font-medium">
          <Shield className="h-4 w-4" />
          Incident Roles
        </h3>
        <Button
          size="sm"
          variant="outline"
          onClick={() => setAdding(true)}
          disabled={adding}
        >
          <Plus className="mr-1 h-3 w-3" />
          Assign
        </Button>
      </div>

      {adding && (
        <div className="flex items-center gap-2 rounded border p-2">
          <Select
            value={newRole}
            onChange={(e) => setNewRole(e.target.value as IncidentRoleType)}
            className="w-44"
          >
            {INCIDENT_ROLES.map((r) => (
              <option key={r} value={r}>
                {r}
              </option>
            ))}
          </Select>
          <Input
            value={newAssignee}
            onChange={(e) => setNewAssignee(e.target.value)}
            placeholder="Person name"
            className="flex-1"
            onKeyDown={(e) => e.key === "Enter" && handleAssign()}
          />
          <Button
            size="sm"
            onClick={handleAssign}
            disabled={!newAssignee.trim() || assignRole.isPending}
          >
            Add
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => {
              setAdding(false);
              setNewAssignee("");
            }}
          >
            Cancel
          </Button>
        </div>
      )}

      {Object.keys(grouped).length === 0 && !adding && (
        <p className="text-sm text-muted-foreground">
          No roles assigned yet.
        </p>
      )}

      <div className="space-y-2">
        {INCIDENT_ROLES.filter((r) => grouped[r]?.length).map((roleName) => (
          <div key={roleName} className="space-y-1">
            <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
              {roleName}
            </p>
            {grouped[roleName]!.map((assignment) => (
              <div
                key={assignment.id}
                className="flex items-center justify-between rounded border px-3 py-1.5"
              >
                <div className="flex items-center gap-2">
                  <Badge variant="outline" className="text-[10px] font-mono">
                    {ROLE_ICONS[assignment.role] ?? "?"}
                  </Badge>
                  <UserCircle className="h-3.5 w-3.5 text-muted-foreground" />
                  <span className="text-sm">{assignment.assignee}</span>
                  {assignment.is_primary && (
                    <Badge variant="secondary" className="text-[9px]">
                      primary
                    </Badge>
                  )}
                </div>
                <Button
                  size="icon"
                  variant="ghost"
                  className="h-6 w-6"
                  onClick={() => handleUnassign(assignment.id)}
                  disabled={unassignRole.isPending}
                >
                  <X className="h-3 w-3 text-destructive" />
                </Button>
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
