import { useState } from "react";
import { AlertTriangle, BarChart3, FileText, Settings, ArrowRight, Check } from "lucide-react";
import { useActiveServices } from "@/hooks/use-services";
import { useQuarters } from "@/hooks/use-quarters";
import { useCompleteOnboarding } from "@/hooks/use-onboarding";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

interface OnboardingWizardProps {
  open: boolean;
  onComplete: () => void;
}

const STEPS = [
  {
    title: "Welcome to Incident Manager",
    description:
      "Track incidents, analyze trends, and generate quarterly reports — all in one place.",
    icon: AlertTriangle,
  },
  {
    title: "Services",
    description:
      "Services represent the systems you monitor. 15 default services are pre-configured. You can add, edit, or disable them in Settings.",
    icon: Settings,
  },
  {
    title: "Dashboard & Metrics",
    description:
      "The dashboard shows MTTR, MTTA, incident heatmaps, and trend analysis. Create incidents to start seeing your metrics.",
    icon: BarChart3,
  },
  {
    title: "Reports",
    description:
      "Generate quarterly DOCX reports with executive summaries, breakdowns by service, and trend analysis. Configure quarters in Settings first.",
    icon: FileText,
  },
] as const;

export function OnboardingWizard({ open, onComplete }: OnboardingWizardProps) {
  const [step, setStep] = useState(0);
  const { data: services } = useActiveServices();
  const { data: quarters } = useQuarters();
  const completeMutation = useCompleteOnboarding();

  const currentStep = STEPS[step];
  const Icon = currentStep.icon;
  const isLastStep = step === STEPS.length - 1;

  const handleNext = async () => {
    if (isLastStep) {
      await completeMutation.mutateAsync();
      onComplete();
    } else {
      setStep((s) => s + 1);
    }
  };

  return (
    <Dialog open={open} onOpenChange={() => {}}>
      <DialogContent className="sm:max-w-md [&>button]:hidden">
        <DialogHeader className="items-center text-center">
          <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-primary/10 mb-2">
            <Icon className="h-6 w-6 text-primary" />
          </div>
          <DialogTitle>{currentStep.title}</DialogTitle>
          <DialogDescription className="text-center">
            {currentStep.description}
          </DialogDescription>
        </DialogHeader>

        {/* Step-specific content */}
        {step === 1 && services && (
          <div className="rounded-md border p-3 text-sm text-muted-foreground max-h-32 overflow-y-auto">
            {services.length} active services configured:{" "}
            {services.slice(0, 5).map((s) => s.name).join(", ")}
            {services.length > 5 && `, +${services.length - 5} more`}
          </div>
        )}

        {step === 3 && quarters && (
          <div className="rounded-md border p-3 text-sm text-muted-foreground">
            {quarters.length > 0
              ? `${quarters.length} quarter(s) configured: ${quarters.map((q) => q.label).join(", ")}`
              : "No quarters configured yet. Set them up in Settings > Quarters."}
          </div>
        )}

        {/* Progress dots */}
        <div className="flex items-center justify-center gap-1.5 py-2">
          {STEPS.map((_, i) => (
            <div
              key={i}
              className={`h-1.5 rounded-full transition-all ${
                i === step
                  ? "w-6 bg-primary"
                  : i < step
                    ? "w-1.5 bg-primary/50"
                    : "w-1.5 bg-muted-foreground/30"
              }`}
            />
          ))}
        </div>

        <div className="flex justify-between">
          <Button
            variant="ghost"
            size="sm"
            onClick={async () => {
              await completeMutation.mutateAsync();
              onComplete();
            }}
            className="text-muted-foreground"
          >
            Skip
          </Button>
          <Button onClick={handleNext} size="sm">
            {isLastStep ? (
              <>
                <Check className="h-4 w-4" />
                Get Started
              </>
            ) : (
              <>
                Next
                <ArrowRight className="h-4 w-4" />
              </>
            )}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
