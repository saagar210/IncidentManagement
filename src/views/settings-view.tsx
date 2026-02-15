import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Plus, Pencil, Trash2, Check, X, Download, Upload, FileSpreadsheet, ExternalLink } from "lucide-react";
import { useForm } from "react-hook-form";
import { open, save } from "@tauri-apps/plugin-dialog";
import { copyFile } from "@tauri-apps/plugin-fs";
import {
  useServices,
  useCreateService,
  useUpdateService,
  useDeleteService,
} from "@/hooks/use-services";
import { useServiceAliases, useCreateServiceAlias, useDeleteServiceAlias } from "@/hooks/use-service-aliases";
import {
  useQuarters,
  useUpsertQuarter,
  useDeleteQuarter,
} from "@/hooks/use-quarters";
import {
  useImportTemplates,
  useDeleteTemplate,
  useExportAllData,
  useImportBackup,
} from "@/hooks/use-import";
import {
  useCustomFields,
  useCreateCustomField,
  useUpdateCustomField,
  useDeleteCustomField,
} from "@/hooks/use-custom-fields";
import {
  useSlaDefinitions,
  useCreateSlaDefinition,
  useUpdateSlaDefinition,
  useDeleteSlaDefinition,
} from "@/hooks/use-sla";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { CSVImportDialog } from "@/components/import/csv-import-dialog";
import { toast } from "@/components/ui/use-toast";
import {
  SEVERITY_LEVELS,
  IMPACT_LEVELS,
  PRIORITY_LEVELS,
  SERVICE_CATEGORIES,
  SERVICE_TIERS,
  TIER_COLORS,
} from "@/lib/constants";
import type {
  Service,
  CreateServiceRequest,
  UpdateServiceRequest,
  QuarterConfig,
  UpsertQuarterRequest,
} from "@/types/incident";
import type {
  CustomFieldDefinition,
  CreateCustomFieldRequest,
  UpdateCustomFieldRequest,
} from "@/types/custom-fields";
import type { SlaDefinition } from "@/types/sla";
import { OllamaConfig } from "@/components/settings/ollama-config";
import { BackupConfig } from "@/components/settings/backup-config";
import { tauriInvoke } from "@/lib/tauri";
import type { ServiceAlias } from "@/types/service-aliases";

// ===================== Services Tab =====================

interface ServiceFormData {
  name: string;
  category: string;
  tier: string;
  default_severity: string;
  default_impact: string;
  description: string;
}

function ServicesTab() {
  const navigate = useNavigate();
  const { data: services, isLoading } = useServices();
  const createService = useCreateService();
  const updateService = useUpdateService();
  const deleteService = useDeleteService();

  const [showAdd, setShowAdd] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);

  const addForm = useForm<ServiceFormData>({
    defaultValues: {
      name: "",
      category: "Other",
      tier: "T3",
      default_severity: "Medium",
      default_impact: "Medium",
      description: "",
    },
  });

  const editForm = useForm<ServiceFormData>();

  const handleAddSubmit = async (data: ServiceFormData) => {
    const req: CreateServiceRequest = {
      name: data.name,
      category: data.category,
      tier: data.tier,
      default_severity: data.default_severity,
      default_impact: data.default_impact,
      description: data.description || undefined,
    };
    try {
      await createService.mutateAsync(req);
      addForm.reset();
      setShowAdd(false);
    } catch {
      // Error is handled by global mutation error handler
    }
  };

  const startEdit = (service: Service) => {
    setEditingId(service.id);
    editForm.reset({
      name: service.name,
      category: service.category,
      tier: service.tier,
      default_severity: service.default_severity,
      default_impact: service.default_impact,
      description: service.description ?? "",
    });
  };

  const handleEditSubmit = async (data: ServiceFormData) => {
    if (!editingId) return;
    const req: UpdateServiceRequest = {
      name: data.name,
      category: data.category,
      tier: data.tier,
      default_severity: data.default_severity,
      default_impact: data.default_impact,
      description: data.description || undefined,
    };
    try {
      await updateService.mutateAsync({ id: editingId, service: req });
      setEditingId(null);
    } catch {
      // Error is handled by global mutation error handler
    }
  };

  const handleToggleActive = async (service: Service) => {
    try {
      await updateService.mutateAsync({
        id: service.id,
        service: { is_active: !service.is_active },
      });
    } catch {
      // Error is handled by global mutation error handler
    }
  };

  const handleDelete = async (service: Service) => {
    const confirmed = window.confirm(
      `Delete service "${service.name}"? This will fail if incidents reference it.`
    );
    if (!confirmed) return;
    try {
      await deleteService.mutateAsync(service.id);
    } catch {
      // Error is handled by global mutation error handler
    }
  };

  if (isLoading) {
    return <p className="text-sm text-muted-foreground">Loading services...</p>;
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Services</h2>
        <Button size="sm" onClick={() => setShowAdd(true)} disabled={showAdd}>
          <Plus className="h-4 w-4" />
          Add Service
        </Button>
      </div>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Category</TableHead>
            <TableHead>Tier</TableHead>
            <TableHead>Default Severity</TableHead>
            <TableHead>Default Impact</TableHead>
            <TableHead>Active</TableHead>
            <TableHead className="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {showAdd && (
            <TableRow>
              <TableCell>
                <Input
                  placeholder="Service name"
                  {...addForm.register("name", { required: true })}
                />
              </TableCell>
              <TableCell>
                <Select {...addForm.register("category")}>
                  {SERVICE_CATEGORIES.map((c) => (
                    <option key={c} value={c}>
                      {c}
                    </option>
                  ))}
                </Select>
              </TableCell>
              <TableCell>
                <Select {...addForm.register("tier")}>
                  {SERVICE_TIERS.map((t) => (
                    <option key={t} value={t}>
                      {t}
                    </option>
                  ))}
                </Select>
              </TableCell>
              <TableCell>
                <Select {...addForm.register("default_severity")}>
                  {SEVERITY_LEVELS.map((s) => (
                    <option key={s} value={s}>
                      {s}
                    </option>
                  ))}
                </Select>
              </TableCell>
              <TableCell>
                <Select {...addForm.register("default_impact")}>
                  {IMPACT_LEVELS.map((s) => (
                    <option key={s} value={s}>
                      {s}
                    </option>
                  ))}
                </Select>
              </TableCell>
              <TableCell>--</TableCell>
              <TableCell className="text-right">
                <div className="flex justify-end gap-1">
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={addForm.handleSubmit(handleAddSubmit)}
                    disabled={createService.isPending}
                  >
                    <Check className="h-4 w-4 text-green-500" />
                  </Button>
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={() => {
                      setShowAdd(false);
                      addForm.reset();
                    }}
                  >
                    <X className="h-4 w-4 text-red-500" />
                  </Button>
                </div>
              </TableCell>
            </TableRow>
          )}
          {services?.map((service) =>
            editingId === service.id ? (
              <TableRow key={service.id}>
                <TableCell>
                  <Input {...editForm.register("name", { required: true })} />
                </TableCell>
                <TableCell>
                  <Select {...editForm.register("category")}>
                    {SERVICE_CATEGORIES.map((c) => (
                      <option key={c} value={c}>
                        {c}
                      </option>
                    ))}
                  </Select>
                </TableCell>
                <TableCell>
                  <Select {...editForm.register("tier")}>
                    {SERVICE_TIERS.map((t) => (
                      <option key={t} value={t}>
                        {t}
                      </option>
                    ))}
                  </Select>
                </TableCell>
                <TableCell>
                  <Select {...editForm.register("default_severity")}>
                    {SEVERITY_LEVELS.map((s) => (
                      <option key={s} value={s}>
                        {s}
                      </option>
                    ))}
                  </Select>
                </TableCell>
                <TableCell>
                  <Select {...editForm.register("default_impact")}>
                    {IMPACT_LEVELS.map((s) => (
                      <option key={s} value={s}>
                        {s}
                      </option>
                    ))}
                  </Select>
                </TableCell>
                <TableCell>
                  <Badge variant={service.is_active ? "default" : "secondary"}>
                    {service.is_active ? "Yes" : "No"}
                  </Badge>
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-1">
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={editForm.handleSubmit(handleEditSubmit)}
                      disabled={updateService.isPending}
                    >
                      <Check className="h-4 w-4 text-green-500" />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => setEditingId(null)}
                    >
                      <X className="h-4 w-4 text-red-500" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            ) : (
              <TableRow key={service.id}>
                <TableCell className="font-medium">
                  <button
                    className="cursor-pointer text-left hover:underline"
                    onClick={() => navigate(`/services/${service.id}`)}
                  >
                    {service.name}
                  </button>
                </TableCell>
                <TableCell>{service.category}</TableCell>
                <TableCell>
                  <Badge
                    variant="outline"
                    className={TIER_COLORS[service.tier as keyof typeof TIER_COLORS] ?? ""}
                  >
                    {service.tier}
                  </Badge>
                </TableCell>
                <TableCell>{service.default_severity}</TableCell>
                <TableCell>{service.default_impact}</TableCell>
                <TableCell>
                  <button
                    onClick={() => handleToggleActive(service)}
                    className="cursor-pointer"
                  >
                    <Badge variant={service.is_active ? "default" : "secondary"}>
                      {service.is_active ? "Yes" : "No"}
                    </Badge>
                  </button>
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-1">
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => navigate(`/services/${service.id}`)}
                      title="View details"
                    >
                      <ExternalLink className="h-4 w-4" />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => startEdit(service)}
                    >
                      <Pencil className="h-4 w-4" />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => handleDelete(service)}
                      disabled={deleteService.isPending}
                    >
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            )
          )}
          {(!services || services.length === 0) && !showAdd && (
            <TableRow>
              <TableCell colSpan={7} className="text-center text-muted-foreground">
                No services configured. Add one to get started.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

// ===================== Quarters Tab =====================

interface QuarterFormData {
  fiscal_year: number;
  quarter_number: number;
  start_date: string;
  end_date: string;
  label: string;
}

function QuartersTab() {
  const { data: quarters, isLoading } = useQuarters();
  const upsertQuarter = useUpsertQuarter();
  const deleteQuarter = useDeleteQuarter();

  const [showAdd, setShowAdd] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);

  const addForm = useForm<QuarterFormData>({
    defaultValues: {
      fiscal_year: new Date().getFullYear(),
      quarter_number: 1,
      start_date: "",
      end_date: "",
      label: "",
    },
  });

  const editForm = useForm<QuarterFormData>();

  const handleAddSubmit = async (data: QuarterFormData) => {
    const req: UpsertQuarterRequest = {
      fiscal_year: data.fiscal_year,
      quarter_number: data.quarter_number,
      start_date: data.start_date,
      end_date: data.end_date,
      label: data.label || `FY${data.fiscal_year} Q${data.quarter_number}`,
    };
    try {
      await upsertQuarter.mutateAsync(req);
      addForm.reset();
      setShowAdd(false);
    } catch {
      // Error is handled by global mutation error handler
    }
  };

  const startEdit = (quarter: QuarterConfig) => {
    setEditingId(quarter.id);
    editForm.reset({
      fiscal_year: quarter.fiscal_year,
      quarter_number: quarter.quarter_number,
      start_date: quarter.start_date.split("T")[0],
      end_date: quarter.end_date.split("T")[0],
      label: quarter.label,
    });
  };

  const handleEditSubmit = async (data: QuarterFormData) => {
    if (!editingId) return;
    const req: UpsertQuarterRequest = {
      id: editingId,
      fiscal_year: data.fiscal_year,
      quarter_number: data.quarter_number,
      start_date: data.start_date,
      end_date: data.end_date,
      label: data.label || `FY${data.fiscal_year} Q${data.quarter_number}`,
    };
    try {
      await upsertQuarter.mutateAsync(req);
      setEditingId(null);
    } catch {
      // Error is handled by global mutation error handler
    }
  };

  const handleDelete = async (quarter: QuarterConfig) => {
    const confirmed = window.confirm(
      `Delete quarter "${quarter.label}"? This cannot be undone.`
    );
    if (!confirmed) return;
    try {
      await deleteQuarter.mutateAsync(quarter.id);
    } catch {
      // Error is handled by global mutation error handler
    }
  };

  if (isLoading) {
    return <p className="text-sm text-muted-foreground">Loading quarters...</p>;
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Quarter Configurations</h2>
        <Button size="sm" onClick={() => setShowAdd(true)} disabled={showAdd}>
          <Plus className="h-4 w-4" />
          Add Quarter
        </Button>
      </div>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Label</TableHead>
            <TableHead>Fiscal Year</TableHead>
            <TableHead>Quarter</TableHead>
            <TableHead>Start Date</TableHead>
            <TableHead>End Date</TableHead>
            <TableHead className="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {showAdd && (
            <TableRow>
              <TableCell>
                <Input
                  placeholder="e.g., FY2025 Q1"
                  {...addForm.register("label")}
                />
              </TableCell>
              <TableCell>
                <Input
                  type="number"
                  min={2000}
                  max={2100}
                  {...addForm.register("fiscal_year", { valueAsNumber: true })}
                />
              </TableCell>
              <TableCell>
                <Select {...addForm.register("quarter_number", { valueAsNumber: true })}>
                  <option value={1}>Q1</option>
                  <option value={2}>Q2</option>
                  <option value={3}>Q3</option>
                  <option value={4}>Q4</option>
                </Select>
              </TableCell>
              <TableCell>
                <Input
                  type="date"
                  {...addForm.register("start_date", { required: true })}
                />
              </TableCell>
              <TableCell>
                <Input
                  type="date"
                  {...addForm.register("end_date", { required: true })}
                />
              </TableCell>
              <TableCell className="text-right">
                <div className="flex justify-end gap-1">
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={addForm.handleSubmit(handleAddSubmit)}
                    disabled={upsertQuarter.isPending}
                  >
                    <Check className="h-4 w-4 text-green-500" />
                  </Button>
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={() => {
                      setShowAdd(false);
                      addForm.reset();
                    }}
                  >
                    <X className="h-4 w-4 text-red-500" />
                  </Button>
                </div>
              </TableCell>
            </TableRow>
          )}
          {quarters?.map((quarter) =>
            editingId === quarter.id ? (
              <TableRow key={quarter.id}>
                <TableCell>
                  <Input {...editForm.register("label")} />
                </TableCell>
                <TableCell>
                  <Input
                    type="number"
                    min={2000}
                    max={2100}
                    {...editForm.register("fiscal_year", { valueAsNumber: true })}
                  />
                </TableCell>
                <TableCell>
                  <Select {...editForm.register("quarter_number", { valueAsNumber: true })}>
                    <option value={1}>Q1</option>
                    <option value={2}>Q2</option>
                    <option value={3}>Q3</option>
                    <option value={4}>Q4</option>
                  </Select>
                </TableCell>
                <TableCell>
                  <Input type="date" {...editForm.register("start_date")} />
                </TableCell>
                <TableCell>
                  <Input type="date" {...editForm.register("end_date")} />
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-1">
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={editForm.handleSubmit(handleEditSubmit)}
                      disabled={upsertQuarter.isPending}
                    >
                      <Check className="h-4 w-4 text-green-500" />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => setEditingId(null)}
                    >
                      <X className="h-4 w-4 text-red-500" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            ) : (
              <TableRow key={quarter.id}>
                <TableCell className="font-medium">{quarter.label}</TableCell>
                <TableCell>{quarter.fiscal_year}</TableCell>
                <TableCell>Q{quarter.quarter_number}</TableCell>
                <TableCell>{quarter.start_date.split("T")[0]}</TableCell>
                <TableCell>{quarter.end_date.split("T")[0]}</TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-1">
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => startEdit(quarter)}
                    >
                      <Pencil className="h-4 w-4" />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => handleDelete(quarter)}
                      disabled={deleteQuarter.isPending}
                    >
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            )
          )}
          {(!quarters || quarters.length === 0) && !showAdd && (
            <TableRow>
              <TableCell colSpan={6} className="text-center text-muted-foreground">
                No quarters configured. Add one to define reporting periods.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

// ===================== Import & Data Tab =====================

function ImportDataTab() {
  const { data: templates, isLoading: templatesLoading } = useImportTemplates();
  const deleteTemplate = useDeleteTemplate();
  const exportData = useExportAllData();
  const importBackup = useImportBackup();
  const [csvImportOpen, setCsvImportOpen] = useState(false);

  const handleExport = async () => {
    try {
      const tempPath = await exportData.mutateAsync();
      // Let user pick where to save
      const savePath = await save({
        defaultPath: "incident_backup.json",
        filters: [{ name: "JSON Files", extensions: ["json"] }],
      });
      if (savePath) {
        await copyFile(tempPath, savePath);
        await tauriInvoke("delete_temp_file", { tempPath });
        toast({
          title: "Export complete",
          description: `Backup saved to ${savePath}`,
        });
      } else {
        await tauriInvoke("delete_temp_file", { tempPath });
      }
    } catch (err) {
      toast({
        title: "Export failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleImportBackup = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "JSON Files", extensions: ["json"] }],
      });
      if (!selected) return;
      const filePath = selected;
      const result = await importBackup.mutateAsync(filePath);

      const parts: string[] = [];
      if (result.services > 0) parts.push(`${result.services} services`);
      if (result.incidents > 0) parts.push(`${result.incidents} incidents`);
      if (result.action_items > 0) parts.push(`${result.action_items} action items`);
      if (result.quarter_configs > 0) parts.push(`${result.quarter_configs} quarters`);
      if (result.custom_field_definitions > 0) {
        parts.push(`${result.custom_field_definitions} custom fields`);
      }
      if (result.custom_field_values > 0) {
        parts.push(`${result.custom_field_values} custom field values`);
      }
      if (result.settings > 0) parts.push(`${result.settings} settings`);

      toast({
        title: "Import complete",
        description: parts.length > 0 ? `Imported: ${parts.join(", ")}` : "No new data imported",
      });

      if (result.errors.length > 0) {
        toast({
          title: "Import warnings",
          description: `${result.errors.length} error(s) occurred during import`,
          variant: "destructive",
        });
      }
    } catch (err) {
      toast({
        title: "Import failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleDeleteTemplate = async (id: string, name: string) => {
    const confirmed = window.confirm(
      `Delete template "${name}"? This cannot be undone.`
    );
    if (!confirmed) return;
    try {
      await deleteTemplate.mutateAsync(id);
    } catch {
      // Error is handled by global mutation error handler
    }
  };

  return (
    <div className="space-y-6">
      {/* CSV Import */}
      <div className="space-y-3">
        <h2 className="text-lg font-semibold">CSV Import</h2>
        <p className="text-sm text-muted-foreground">
          Import incidents from a CSV file with column mapping.
        </p>
        <Button onClick={() => setCsvImportOpen(true)}>
          <FileSpreadsheet className="h-4 w-4" />
          Import from CSV
        </Button>
        <CSVImportDialog
          open={csvImportOpen}
          onOpenChange={setCsvImportOpen}
        />
      </div>

      {/* Saved Templates */}
      <div className="space-y-3">
        <h2 className="text-lg font-semibold">Saved Import Templates</h2>
        {templatesLoading ? (
          <p className="text-sm text-muted-foreground">Loading templates...</p>
        ) : templates && templates.length > 0 ? (
          <div className="space-y-1">
            {templates.map((tpl) => (
              <div
                key={tpl.id}
                className="flex items-center justify-between rounded border px-3 py-2"
              >
                <div>
                  <span className="text-sm font-medium">{tpl.name}</span>
                  <span className="ml-2 text-xs text-muted-foreground">
                    Created {tpl.created_at.split("T")[0]}
                  </span>
                </div>
                <Button
                  size="icon"
                  variant="ghost"
                  onClick={() => handleDeleteTemplate(tpl.id, tpl.name)}
                  disabled={deleteTemplate.isPending}
                >
                  <Trash2 className="h-4 w-4 text-destructive" />
                </Button>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-sm text-muted-foreground">
            No saved templates. Save one during CSV import.
          </p>
        )}
      </div>

      {/* Data Backup */}
      <div className="space-y-3">
        <h2 className="text-lg font-semibold">Data Backup</h2>
        <p className="text-sm text-muted-foreground">
          Export all data as JSON or restore from a backup.
        </p>
        <div className="flex gap-3">
          <Button
            variant="outline"
            onClick={handleExport}
            disabled={exportData.isPending}
          >
            <Download className="h-4 w-4" />
            {exportData.isPending ? "Exporting..." : "Export All Data"}
          </Button>
          <Button
            variant="outline"
            onClick={handleImportBackup}
            disabled={importBackup.isPending}
          >
            <Upload className="h-4 w-4" />
            {importBackup.isPending ? "Importing..." : "Import Backup"}
          </Button>
        </div>
      </div>
    </div>
  );
}

// ===================== Custom Fields Tab =====================

interface CustomFieldFormData {
  name: string;
  field_type: "text" | "number" | "select";
  options: string;
  display_order: number;
}

function CustomFieldsTab() {
  const { data: fields, isLoading } = useCustomFields();
  const createField = useCreateCustomField();
  const updateField = useUpdateCustomField();
  const deleteField = useDeleteCustomField();

  const [showAdd, setShowAdd] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);

  const addForm = useForm<CustomFieldFormData>({
    defaultValues: {
      name: "",
      field_type: "text",
      options: "",
      display_order: 0,
    },
  });

  const editForm = useForm<CustomFieldFormData>();

  const handleAddSubmit = async (data: CustomFieldFormData) => {
    const req: CreateCustomFieldRequest = {
      name: data.name,
      field_type: data.field_type,
      options: data.field_type === "select" ? data.options : undefined,
      display_order: data.display_order,
    };
    try {
      await createField.mutateAsync(req);
      addForm.reset();
      setShowAdd(false);
    } catch {
      // Error handled by global mutation handler
    }
  };

  const startEdit = (field: CustomFieldDefinition) => {
    setEditingId(field.id);
    editForm.reset({
      name: field.name,
      field_type: field.field_type,
      options: field.options,
      display_order: field.display_order,
    });
  };

  const handleEditSubmit = async (data: CustomFieldFormData) => {
    if (!editingId) return;
    const req: UpdateCustomFieldRequest = {
      name: data.name,
      field_type: data.field_type,
      options: data.field_type === "select" ? data.options : undefined,
      display_order: data.display_order,
    };
    try {
      await updateField.mutateAsync({ id: editingId, field: req });
      setEditingId(null);
    } catch {
      // Error handled by global mutation handler
    }
  };

  const handleDelete = async (field: CustomFieldDefinition) => {
    const confirmed = window.confirm(
      `Delete custom field "${field.name}"? Values for this field will be lost.`
    );
    if (!confirmed) return;
    try {
      await deleteField.mutateAsync(field.id);
    } catch {
      // Error handled by global mutation handler
    }
  };

  if (isLoading) {
    return <p className="text-sm text-muted-foreground">Loading custom fields...</p>;
  }

  const watchedAddType = addForm.watch("field_type");
  const watchedEditType = editForm.watch("field_type");

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">Custom Fields</h2>
          <p className="text-sm text-muted-foreground">
            Define additional fields for incidents.
          </p>
        </div>
        <Button size="sm" onClick={() => setShowAdd(true)} disabled={showAdd}>
          <Plus className="h-4 w-4" />
          Add Field
        </Button>
      </div>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Options (for select)</TableHead>
            <TableHead>Order</TableHead>
            <TableHead className="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {showAdd && (
            <TableRow>
              <TableCell>
                <Input
                  placeholder="Field name"
                  {...addForm.register("name", { required: true })}
                />
              </TableCell>
              <TableCell>
                <Select {...addForm.register("field_type")}>
                  <option value="text">Text</option>
                  <option value="number">Number</option>
                  <option value="select">Select</option>
                </Select>
              </TableCell>
              <TableCell>
                {watchedAddType === "select" && (
                  <Input
                    placeholder="Comma-separated options"
                    {...addForm.register("options")}
                  />
                )}
              </TableCell>
              <TableCell>
                <Input
                  type="number"
                  className="w-20"
                  {...addForm.register("display_order", { valueAsNumber: true })}
                />
              </TableCell>
              <TableCell className="text-right">
                <div className="flex justify-end gap-1">
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={addForm.handleSubmit(handleAddSubmit)}
                    disabled={createField.isPending}
                  >
                    <Check className="h-4 w-4 text-green-500" />
                  </Button>
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={() => {
                      setShowAdd(false);
                      addForm.reset();
                    }}
                  >
                    <X className="h-4 w-4 text-red-500" />
                  </Button>
                </div>
              </TableCell>
            </TableRow>
          )}
          {fields?.map((field) =>
            editingId === field.id ? (
              <TableRow key={field.id}>
                <TableCell>
                  <Input {...editForm.register("name", { required: true })} />
                </TableCell>
                <TableCell>
                  <Select {...editForm.register("field_type")}>
                    <option value="text">Text</option>
                    <option value="number">Number</option>
                    <option value="select">Select</option>
                  </Select>
                </TableCell>
                <TableCell>
                  {watchedEditType === "select" && (
                    <Input
                      placeholder="Comma-separated options"
                      {...editForm.register("options")}
                    />
                  )}
                </TableCell>
                <TableCell>
                  <Input
                    type="number"
                    className="w-20"
                    {...editForm.register("display_order", { valueAsNumber: true })}
                  />
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-1">
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={editForm.handleSubmit(handleEditSubmit)}
                      disabled={updateField.isPending}
                    >
                      <Check className="h-4 w-4 text-green-500" />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => setEditingId(null)}
                    >
                      <X className="h-4 w-4 text-red-500" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            ) : (
              <TableRow key={field.id}>
                <TableCell className="font-medium">{field.name}</TableCell>
                <TableCell>
                  <Badge variant="outline">{field.field_type}</Badge>
                </TableCell>
                <TableCell className="text-sm text-muted-foreground">
                  {field.field_type === "select" ? field.options : "--"}
                </TableCell>
                <TableCell>{field.display_order}</TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-1">
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => startEdit(field)}
                    >
                      <Pencil className="h-4 w-4" />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => handleDelete(field)}
                      disabled={deleteField.isPending}
                    >
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            )
          )}
          {(!fields || fields.length === 0) && !showAdd && (
            <TableRow>
              <TableCell colSpan={5} className="text-center text-muted-foreground">
                No custom fields defined. Add one to extend incident data.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

// ===================== SLA Tab =====================

interface SlaFormData {
  name: string;
  priority: string;
  response_time_minutes: number;
  resolve_time_minutes: number;
}

function formatSlaTime(minutes: number): string {
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (hours < 24) return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
  const days = Math.floor(hours / 24);
  const remainingHours = hours % 24;
  return remainingHours > 0 ? `${days}d ${remainingHours}h` : `${days}d`;
}

function SlaTab() {
  const { data: definitions, isLoading } = useSlaDefinitions();
  const createSla = useCreateSlaDefinition();
  const updateSla = useUpdateSlaDefinition();
  const deleteSla = useDeleteSlaDefinition();

  const [showAdd, setShowAdd] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);

  const addForm = useForm<SlaFormData>({
    defaultValues: {
      name: "",
      priority: "P0",
      response_time_minutes: 15,
      resolve_time_minutes: 60,
    },
  });

  const editForm = useForm<SlaFormData>();

  const handleAddSubmit = async (data: SlaFormData) => {
    try {
      await createSla.mutateAsync(data);
      addForm.reset();
      setShowAdd(false);
    } catch (err) {
      toast({
        title: "Failed to create SLA",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const startEdit = (def: SlaDefinition) => {
    setEditingId(def.id);
    editForm.reset({
      name: def.name,
      priority: def.priority,
      response_time_minutes: def.response_time_minutes,
      resolve_time_minutes: def.resolve_time_minutes,
    });
  };

  const handleEditSubmit = async (data: SlaFormData) => {
    if (!editingId) return;
    try {
      await updateSla.mutateAsync({
        id: editingId,
        req: {
          name: data.name,
          priority: data.priority,
          response_time_minutes: data.response_time_minutes,
          resolve_time_minutes: data.resolve_time_minutes,
        },
      });
      setEditingId(null);
    } catch (err) {
      toast({
        title: "Failed to update SLA",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleDelete = async (def: SlaDefinition) => {
    const confirmed = window.confirm(
      `Delete SLA definition "${def.name}"? This cannot be undone.`
    );
    if (!confirmed) return;
    try {
      await deleteSla.mutateAsync(def.id);
    } catch (err) {
      toast({
        title: "Failed to delete SLA",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleToggleActive = async (def: SlaDefinition) => {
    try {
      await updateSla.mutateAsync({
        id: def.id,
        req: { is_active: !def.is_active },
      });
    } catch (err) {
      toast({
        title: "Failed to toggle SLA",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  if (isLoading) {
    return <p className="text-sm text-muted-foreground">Loading SLA definitions...</p>;
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">SLA Definitions</h2>
          <p className="text-sm text-muted-foreground">
            Define response and resolution time targets per priority level.
          </p>
        </div>
        <Button size="sm" onClick={() => setShowAdd(true)} disabled={showAdd}>
          <Plus className="h-4 w-4" />
          Add SLA
        </Button>
      </div>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Priority</TableHead>
            <TableHead>Response Target</TableHead>
            <TableHead>Resolve Target</TableHead>
            <TableHead>Active</TableHead>
            <TableHead className="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {showAdd && (
            <TableRow>
              <TableCell>
                <Input
                  placeholder="SLA name"
                  {...addForm.register("name", { required: true })}
                />
              </TableCell>
              <TableCell>
                <Select {...addForm.register("priority")}>
                  {PRIORITY_LEVELS.map((p) => (
                    <option key={p} value={p}>{p}</option>
                  ))}
                </Select>
              </TableCell>
              <TableCell>
                <Input
                  type="number"
                  min={1}
                  className="w-24"
                  {...addForm.register("response_time_minutes", {
                    valueAsNumber: true,
                    required: true,
                  })}
                />
                <span className="ml-1 text-xs text-muted-foreground">min</span>
              </TableCell>
              <TableCell>
                <Input
                  type="number"
                  min={1}
                  className="w-24"
                  {...addForm.register("resolve_time_minutes", {
                    valueAsNumber: true,
                    required: true,
                  })}
                />
                <span className="ml-1 text-xs text-muted-foreground">min</span>
              </TableCell>
              <TableCell />
              <TableCell className="text-right">
                <div className="flex justify-end gap-1">
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={addForm.handleSubmit(handleAddSubmit)}
                    disabled={createSla.isPending}
                  >
                    <Check className="h-4 w-4 text-green-500" />
                  </Button>
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={() => {
                      setShowAdd(false);
                      addForm.reset();
                    }}
                  >
                    <X className="h-4 w-4 text-red-500" />
                  </Button>
                </div>
              </TableCell>
            </TableRow>
          )}
          {definitions?.map((def) =>
            editingId === def.id ? (
              <TableRow key={def.id}>
                <TableCell>
                  <Input {...editForm.register("name", { required: true })} />
                </TableCell>
                <TableCell>
                  <Select {...editForm.register("priority")}>
                    {PRIORITY_LEVELS.map((p) => (
                      <option key={p} value={p}>{p}</option>
                    ))}
                  </Select>
                </TableCell>
                <TableCell>
                  <Input
                    type="number"
                    min={1}
                    className="w-24"
                    {...editForm.register("response_time_minutes", {
                      valueAsNumber: true,
                    })}
                  />
                  <span className="ml-1 text-xs text-muted-foreground">min</span>
                </TableCell>
                <TableCell>
                  <Input
                    type="number"
                    min={1}
                    className="w-24"
                    {...editForm.register("resolve_time_minutes", {
                      valueAsNumber: true,
                    })}
                  />
                  <span className="ml-1 text-xs text-muted-foreground">min</span>
                </TableCell>
                <TableCell />
                <TableCell className="text-right">
                  <div className="flex justify-end gap-1">
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={editForm.handleSubmit(handleEditSubmit)}
                      disabled={updateSla.isPending}
                    >
                      <Check className="h-4 w-4 text-green-500" />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => setEditingId(null)}
                    >
                      <X className="h-4 w-4 text-red-500" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            ) : (
              <TableRow key={def.id} className={!def.is_active ? "opacity-50" : ""}>
                <TableCell className="font-medium">{def.name}</TableCell>
                <TableCell>
                  <Badge variant="outline">{def.priority}</Badge>
                </TableCell>
                <TableCell>{formatSlaTime(def.response_time_minutes)}</TableCell>
                <TableCell>{formatSlaTime(def.resolve_time_minutes)}</TableCell>
                <TableCell>
                  <button
                    onClick={() => handleToggleActive(def)}
                    className={`h-4 w-4 rounded border ${
                      def.is_active
                        ? "bg-green-500 border-green-500"
                        : "border-muted-foreground"
                    }`}
                  />
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-1">
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => startEdit(def)}
                    >
                      <Pencil className="h-4 w-4" />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => handleDelete(def)}
                      disabled={deleteSla.isPending}
                    >
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            )
          )}
          {(!definitions || definitions.length === 0) && !showAdd && (
            <TableRow>
              <TableCell colSpan={6} className="text-center text-muted-foreground">
                No SLA definitions. Add one to set response/resolve time targets.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

// ===================== Service Aliases Tab =====================

function ServiceAliasesTab() {
  const { data: services, isLoading: servicesLoading } = useServices();
  const { data: aliases, isLoading: aliasesLoading } = useServiceAliases();
  const createAlias = useCreateServiceAlias();
  const deleteAlias = useDeleteServiceAlias();

  const [aliasText, setAliasText] = useState("");
  const [serviceId, setServiceId] = useState<string>("");

  const add = async () => {
    if (!aliasText.trim() || !serviceId) return;
    try {
      await createAlias.mutateAsync({ alias: aliasText.trim(), service_id: serviceId });
      setAliasText("");
      setServiceId("");
      toast({ title: "Alias created" });
    } catch (err) {
      toast({ title: "Failed to create alias", description: String(err), variant: "destructive" });
    }
  };

  const remove = async (a: ServiceAlias) => {
    const confirmed = window.confirm(`Delete alias "${a.alias}"?`);
    if (!confirmed) return;
    try {
      await deleteAlias.mutateAsync(a.id);
    } catch (err) {
      toast({ title: "Failed to delete alias", description: String(err), variant: "destructive" });
    }
  };

  if (servicesLoading || aliasesLoading) {
    return <p className="text-sm text-muted-foreground">Loading service aliases...</p>;
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">Service Aliases</h2>
          <p className="text-sm text-muted-foreground">
            Map import names to canonical services so CSV imports are deterministic.
          </p>
        </div>
      </div>

      <div className="rounded border p-4 space-y-3">
        <div className="grid grid-cols-3 gap-3">
          <div className="space-y-1">
            <label className="text-sm font-medium">Alias</label>
            <Input value={aliasText} onChange={(e) => setAliasText(e.target.value)} placeholder="Jira service name..." />
          </div>
          <div className="space-y-1">
            <label className="text-sm font-medium">Canonical Service</label>
            <Select value={serviceId} onChange={(e) => setServiceId(e.target.value)}>
              <option value="">Select a service...</option>
              {services?.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.name}
                </option>
              ))}
            </Select>
          </div>
          <div className="flex items-end">
            <Button onClick={add} disabled={createAlias.isPending || !aliasText.trim() || !serviceId}>
              <Plus className="h-4 w-4" />
              Add Alias
            </Button>
          </div>
        </div>
      </div>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Alias</TableHead>
            <TableHead>Service</TableHead>
            <TableHead>Created</TableHead>
            <TableHead className="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {(aliases ?? []).map((a) => (
            <TableRow key={a.id}>
              <TableCell className="font-medium">{a.alias}</TableCell>
              <TableCell>{a.service_name}</TableCell>
              <TableCell className="text-sm text-muted-foreground">{a.created_at}</TableCell>
              <TableCell className="text-right">
                <Button size="icon" variant="ghost" onClick={() => remove(a)} disabled={deleteAlias.isPending}>
                  <Trash2 className="h-4 w-4 text-destructive" />
                </Button>
              </TableCell>
            </TableRow>
          ))}
          {(!aliases || aliases.length === 0) && (
            <TableRow>
              <TableCell colSpan={4} className="text-center text-muted-foreground">
                No aliases yet.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

// ===================== Settings View =====================

type SettingsTab = "services" | "service-aliases" | "quarters" | "custom-fields" | "sla" | "import" | "backup" | "ai";

export function SettingsView() {
  const [activeTab, setActiveTab] = useState<SettingsTab>("services");

  const tabs: { key: SettingsTab; label: string }[] = [
    { key: "services", label: "Services" },
    { key: "service-aliases", label: "Service Aliases" },
    { key: "quarters", label: "Quarters" },
    { key: "custom-fields", label: "Custom Fields" },
    { key: "sla", label: "SLA Targets" },
    { key: "import", label: "Import & Data" },
    { key: "backup", label: "Backup" },
    { key: "ai", label: "AI (Ollama)" },
  ];

  return (
    <div className="space-y-6 p-6">
      <h1 className="text-2xl font-semibold">Settings</h1>

      <div className="border-b">
        <nav className="-mb-px flex gap-4">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`border-b-2 px-1 pb-2 text-sm font-medium transition-colors ${
                activeTab === tab.key
                  ? "border-primary text-foreground"
                  : "border-transparent text-muted-foreground hover:text-foreground"
              }`}
            >
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      {activeTab === "services" && <ServicesTab />}
      {activeTab === "service-aliases" && <ServiceAliasesTab />}
      {activeTab === "quarters" && <QuartersTab />}
      {activeTab === "custom-fields" && <CustomFieldsTab />}
      {activeTab === "sla" && <SlaTab />}
      {activeTab === "import" && <ImportDataTab />}
      {activeTab === "backup" && <BackupConfig />}
      {activeTab === "ai" && <OllamaConfig />}
    </div>
  );
}
