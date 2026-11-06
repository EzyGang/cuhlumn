import type { ComponentChildren, JSX } from "preact";
import { AlertDialog } from "@base-ui/react/alert-dialog";
import { buttonClasses } from "./buttonClasses";

interface ConfirmDialogProps {
  trigger: ComponentChildren;
  title: string;
  description: string;
  confirmLabel: string;
  onConfirm: () => void;
  disabled?: boolean;
  triggerLabel?: string;
}

type ConfirmPopupProps = Pick<
  ConfirmDialogProps,
  "title" | "description" | "confirmLabel" | "onConfirm"
>;

const ConfirmPopup = ({
  title,
  description,
  confirmLabel,
  onConfirm,
}: ConfirmPopupProps): JSX.Element => (
  <AlertDialog.Popup className="fixed top-1/2 left-1/2 z-50 flex w-md max-w-[calc(100vw-2rem)] -translate-x-1/2 -translate-y-1/2 flex-col gap-5 border border-border bg-card p-5">
    <div class="flex flex-col gap-2">
      <AlertDialog.Title className="text-lg font-semibold">{title}</AlertDialog.Title>
      <AlertDialog.Description className="text-sm text-secondary">
        {description}
      </AlertDialog.Description>
    </div>
    <div class="flex justify-end gap-3">
      <AlertDialog.Close className={buttonClasses("secondary")}>Cancel</AlertDialog.Close>
      <AlertDialog.Close className={buttonClasses("danger")} onClick={onConfirm}>
        {confirmLabel}
      </AlertDialog.Close>
    </div>
  </AlertDialog.Popup>
);

export const ConfirmDialog = ({
  trigger,
  title,
  description,
  confirmLabel,
  onConfirm,
  disabled = false,
  triggerLabel,
}: ConfirmDialogProps): JSX.Element => (
  <AlertDialog.Root>
    <AlertDialog.Trigger
      disabled={disabled}
      aria-label={triggerLabel}
      className={buttonClasses("danger")}
    >
      {trigger}
    </AlertDialog.Trigger>
    <AlertDialog.Portal>
      <AlertDialog.Backdrop className="fixed inset-0 z-40 bg-background/70" />
      <ConfirmPopup
        title={title}
        description={description}
        confirmLabel={confirmLabel}
        onConfirm={onConfirm}
      />
    </AlertDialog.Portal>
  </AlertDialog.Root>
);
