import type { ComponentChildren, JSX } from "preact";
import { Dialog as BaseDialog } from "@base-ui/react/dialog";
import { X } from "lucide-preact";
import { buttonClasses } from "./buttonClasses";

interface DialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  trigger: ComponentChildren;
  title: string;
  children: ComponentChildren;
}

export const Dialog = ({
  open,
  onOpenChange,
  trigger,
  title,
  children,
}: DialogProps): JSX.Element => (
  <BaseDialog.Root open={open} onOpenChange={onOpenChange}>
    <BaseDialog.Trigger className={buttonClasses("secondary")}>{trigger}</BaseDialog.Trigger>
    <BaseDialog.Portal>
      <BaseDialog.Backdrop className="fixed inset-0 z-40 bg-background/70" />
      <BaseDialog.Popup className="fixed top-1/2 left-1/2 z-50 flex max-h-[calc(100dvh-4rem)] w-3xl max-w-[calc(100vw-2rem)] -translate-x-1/2 -translate-y-1/2 flex-col gap-5 overflow-hidden border border-border bg-card p-5">
        <div class="flex items-center justify-between gap-4">
          <BaseDialog.Title className="text-lg font-semibold">{title}</BaseDialog.Title>
          <BaseDialog.Close className={buttonClasses("ghost")} aria-label="Close">
            <X size={16} aria-hidden="true" />
          </BaseDialog.Close>
        </div>
        <div class="min-h-0 overflow-y-auto">{children}</div>
      </BaseDialog.Popup>
    </BaseDialog.Portal>
  </BaseDialog.Root>
);
