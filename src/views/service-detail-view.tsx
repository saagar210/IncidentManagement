import { useState, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, Plus, Trash2, Save, Link2 } from "lucide-react";
import {
  useService,
  useUpdateService,
  useServices,
  useServiceDependencies,
  useServiceDependents,
  useAddServiceDependency,
  useRemoveServiceDependency,
} from "@/hooks/use-services";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { Textarea } from "@/components/ui/textarea";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { RunbookEditor } from "@/components/services/runbook-editor";
import { toast } from "@/components/ui/use-toast";
import {
  SEVERITY_LEVELS,
  IMPACT_LEVELS,
  SERVICE_CATEGORIES,
  SERVICE_TIERS,
  TIER_LABELS,
  TIER_COLORS,
  DEPENDENCY_TYPES,
} from "@/lib/constants";
import type { UpdateServiceRequest } from "@/types/incident";

export function ServiceDetailView() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: service, isLoading } = useService(id);
  const updateService = useUpdateService();
  const { data: allServices } = useServices();
  const { data: dependencies } = useServiceDependencies(id);
  const { data: dependents } = useServiceDependents(id);
  const addDependency = useAddServiceDependency();
  const removeDependency = useRemoveServiceDependency();

  // Form state
  const [name, setName] = useState<string | null>(null);
  const [category, setCategory] = useState<string | null>(null);
  const [defaultSeverity, setDefaultSeverity] = useState<string | null>(null);
  const [defaultImpact, setDefaultImpact] = useState<string | null>(null);
  const [description, setDescription] = useState<string | null>(null);
  const [owner, setOwner] = useState<string | null>(null);
  const [tier, setTier] = useState<string | null>(null);
  const [runbook, setRunbook] = useState<string | null>(null);

  // Dep add form
  const [addingDep, setAddingDep] = useState(false);
  const [newDepServiceId, setNewDepServiceId] = useState("");
  const [newDepType, setNewDepType] = useState("runtime");

  const hasChanges =
    name !== null ||
    category !== null ||
    defaultSeverity !== null ||
    defaultImpact !== null ||
    description !== null ||
    owner !== null ||
    tier !== null ||
    runbook !== null;

  const handleSave = useCallback(async () => {
    if (!id || !hasChanges) return;
    const req: UpdateServiceRequest = {};
    if (name !== null) req.name = name;
    if (category !== null) req.category = category;
    if (defaultSeverity !== null) req.default_severity = defaultSeverity;
    if (defaultImpact !== null) req.default_impact = defaultImpact;
    if (description !== null) req.description = description;
    if (owner !== null) req.owner = owner;
    if (tier !== null) req.tier = tier;
    if (runbook !== null) req.runbook = runbook;

    try {
      await updateService.mutateAsync({ id, service: req });
      // Reset dirty state
      setName(null);
      setCategory(null);
      setDefaultSeverity(null);
      setDefaultImpact(null);
      setDescription(null);
      setOwner(null);
      setTier(null);
      setRunbook(null);
      toast({ title: "Service updated" });
    } catch (err) {
      toast({
        title: "Update failed",
        description: String(err),
        variant: "destructive",
      });
    }
  }, [
    id,
    hasChanges,
    name,
    category,
    defaultSeverity,
    defaultImpact,
    description,
    owner,
    tier,
    runbook,
    updateService,
  ]);

  const handleAddDep = async () => {
    if (!id || !newDepServiceId) return;
    try {
      await addDependency.mutateAsync({
        service_id: id,
        depends_on_service_id: newDepServiceId,
        dependency_type: newDepType,
      });
      setAddingDep(false);
      setNewDepServiceId("");
      setNewDepType("runtime");
      toast({ title: "Dependency added" });
    } catch (err) {
      toast({
        title: "Failed to add dependency",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleRemoveDep = async (depId: string, serviceId: string, dependsOnServiceId: string) => {
    try {
      await removeDependency.mutateAsync({
        id: depId,
        serviceId,
        dependsOnServiceId,
      });
      toast({ title: "Dependency removed" });
    } catch (err) {
      toast({
        title: "Failed to remove dependency",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  if (isLoading) {
    return (
      <div className="p-6">
        <p className="text-muted-foreground">Loading service...</p>
      </div>
    );
  }

  if (!service) {
    return (
      <div className="p-6">
        <p className="text-muted-foreground">Service not found.</p>
        <Button variant="ghost" onClick={() => navigate("/settings")}>
          <ArrowLeft className="mr-1 h-4 w-4" />
          Back to Settings
        </Button>
      </div>
    );
  }

  const currentTier = (tier ?? service.tier) as keyof typeof TIER_COLORS;

  // Exclude self and already-added deps from the available list
  const depIds = new Set(
    (dependencies ?? []).map((d) => d.depends_on_service_id)
  );
  const availableForDep = (allServices ?? []).filter(
    (s) => s.id !== id && !depIds.has(s.id) && s.is_active
  );

  return (
    <div className="space-y-6 p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="icon" onClick={() => navigate("/settings")}>
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <div>
            <h1 className="text-2xl font-semibold">
              {name ?? service.name}
            </h1>
            <div className="flex items-center gap-2 mt-1">
              <Badge
                variant="outline"
                className={TIER_COLORS[currentTier] ?? ""}
              >
                {TIER_LABELS[currentTier] ?? currentTier}
              </Badge>
              <Badge variant={service.is_active ? "default" : "secondary"}>
                {service.is_active ? "Active" : "Inactive"}
              </Badge>
            </div>
          </div>
        </div>
        <Button onClick={handleSave} disabled={!hasChanges || updateService.isPending}>
          <Save className="mr-1 h-4 w-4" />
          {updateService.isPending ? "Saving..." : "Save Changes"}
        </Button>
      </div>

      <Tabs defaultValue="details">
        <TabsList>
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="dependencies">
            Dependencies
            {(dependencies?.length ?? 0) > 0 && (
              <Badge variant="secondary" className="ml-1.5 h-5 min-w-[20px] px-1 text-[10px]">
                {dependencies?.length}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="runbook">Runbook</TabsTrigger>
        </TabsList>

        {/* Details Tab */}
        <TabsContent value="details">
          <Card>
            <CardHeader>
              <CardTitle>Service Configuration</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1.5">
                  <label className="text-sm font-medium">Name</label>
                  <Input
                    value={name ?? service.name}
                    onChange={(e) => setName(e.target.value)}
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-sm font-medium">Owner</label>
                  <Input
                    value={owner ?? service.owner}
                    onChange={(e) => setOwner(e.target.value)}
                    placeholder="Team or person responsible"
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-sm font-medium">Category</label>
                  <Select
                    value={category ?? service.category}
                    onChange={(e) => setCategory(e.target.value)}
                  >
                    {SERVICE_CATEGORIES.map((c) => (
                      <option key={c} value={c}>{c}</option>
                    ))}
                  </Select>
                </div>
                <div className="space-y-1.5">
                  <label className="text-sm font-medium">Tier</label>
                  <Select
                    value={tier ?? service.tier}
                    onChange={(e) => setTier(e.target.value)}
                  >
                    {SERVICE_TIERS.map((t) => (
                      <option key={t} value={t}>{TIER_LABELS[t]}</option>
                    ))}
                  </Select>
                </div>
                <div className="space-y-1.5">
                  <label className="text-sm font-medium">Default Severity</label>
                  <Select
                    value={defaultSeverity ?? service.default_severity}
                    onChange={(e) => setDefaultSeverity(e.target.value)}
                  >
                    {SEVERITY_LEVELS.map((s) => (
                      <option key={s} value={s}>{s}</option>
                    ))}
                  </Select>
                </div>
                <div className="space-y-1.5">
                  <label className="text-sm font-medium">Default Impact</label>
                  <Select
                    value={defaultImpact ?? service.default_impact}
                    onChange={(e) => setDefaultImpact(e.target.value)}
                  >
                    {IMPACT_LEVELS.map((i) => (
                      <option key={i} value={i}>{i}</option>
                    ))}
                  </Select>
                </div>
              </div>
              <div className="space-y-1.5">
                <label className="text-sm font-medium">Description</label>
                <Textarea
                  value={description ?? service.description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="What does this service do?"
                  rows={3}
                />
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Dependencies Tab */}
        <TabsContent value="dependencies">
          <div className="space-y-4">
            {/* Depends On */}
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0">
                <CardTitle className="text-base">Depends On</CardTitle>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setAddingDep(true)}
                  disabled={addingDep || availableForDep.length === 0}
                >
                  <Plus className="mr-1 h-3 w-3" />
                  Add
                </Button>
              </CardHeader>
              <CardContent>
                {addingDep && (
                  <div className="mb-3 flex items-center gap-2 rounded border p-2">
                    <Select
                      value={newDepServiceId}
                      onChange={(e) => setNewDepServiceId(e.target.value)}
                      className="flex-1"
                    >
                      <option value="">Select service...</option>
                      {availableForDep.map((s) => (
                        <option key={s.id} value={s.id}>
                          {s.name}
                        </option>
                      ))}
                    </Select>
                    <Select
                      value={newDepType}
                      onChange={(e) => setNewDepType(e.target.value)}
                    >
                      {DEPENDENCY_TYPES.map((t) => (
                        <option key={t} value={t}>
                          {t}
                        </option>
                      ))}
                    </Select>
                    <Button
                      size="sm"
                      onClick={handleAddDep}
                      disabled={!newDepServiceId || addDependency.isPending}
                    >
                      Add
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => {
                        setAddingDep(false);
                        setNewDepServiceId("");
                      }}
                    >
                      Cancel
                    </Button>
                  </div>
                )}
                {dependencies && dependencies.length > 0 ? (
                  <div className="space-y-1">
                    {dependencies.map((dep) => (
                      <div
                        key={dep.id}
                        className="flex items-center justify-between rounded border px-3 py-2"
                      >
                        <div className="flex items-center gap-2">
                          <Link2 className="h-3.5 w-3.5 text-muted-foreground" />
                          <span className="text-sm font-medium">
                            {dep.depends_on_service_name ?? dep.depends_on_service_id}
                          </span>
                          <Badge variant="outline" className="text-[10px]">
                            {dep.dependency_type}
                          </Badge>
                        </div>
                        <Button
                          size="icon"
                          variant="ghost"
                          onClick={() =>
                            handleRemoveDep(
                              dep.id,
                              dep.service_id,
                              dep.depends_on_service_id
                            )
                          }
                          disabled={removeDependency.isPending}
                        >
                          <Trash2 className="h-3.5 w-3.5 text-destructive" />
                        </Button>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground">
                    No dependencies configured.
                  </p>
                )}
              </CardContent>
            </Card>

            {/* Depended On By */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Depended On By</CardTitle>
              </CardHeader>
              <CardContent>
                {dependents && dependents.length > 0 ? (
                  <div className="space-y-1">
                    {dependents.map((dep) => (
                      <div
                        key={dep.id}
                        className="flex items-center gap-2 rounded border px-3 py-2"
                      >
                        <Link2 className="h-3.5 w-3.5 text-muted-foreground" />
                        <span className="text-sm font-medium">
                          {dep.depends_on_service_name ?? dep.service_id}
                        </span>
                        <Badge variant="outline" className="text-[10px]">
                          {dep.dependency_type}
                        </Badge>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground">
                    No services depend on this one.
                  </p>
                )}
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* Runbook Tab */}
        <TabsContent value="runbook">
          <Card>
            <CardContent className="pt-6">
              <RunbookEditor
                value={runbook ?? service.runbook}
                onChange={setRunbook}
              />
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
