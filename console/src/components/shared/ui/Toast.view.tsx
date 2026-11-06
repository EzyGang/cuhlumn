import type { ComponentChildren, JSX } from "preact";
import { Toast } from "@base-ui/react/toast";
import { X } from "lucide-preact";

export type ToastTone = "error" | "success";

export const toastManager = Toast.createToastManager();

export const showToast = (tone: ToastTone, title: string, description?: string): void => {
  toastManager.add({ type: tone, title, description, priority: tone === "error" ? "high" : "low" });
};

const ToastList = (): JSX.Element => {
  const { toasts } = Toast.useToastManager();
  return (
    <>
      {toasts.map((toast) => (
        <Toast.Root
          key={toast.id}
          toast={toast}
          className="flex items-start gap-3 border border-border bg-card p-4 data-[type=error]:border-error/70 data-[type=success]:border-success/40"
        >
          <Toast.Content className="flex min-w-0 flex-1 flex-col gap-1">
            <Toast.Title className="text-sm font-semibold" />
            <Toast.Description className="text-sm break-words text-secondary" />
          </Toast.Content>
          <Toast.Close aria-label="Dismiss" className="p-1 text-muted hover:text-white">
            <X size={14} aria-hidden="true" />
          </Toast.Close>
        </Toast.Root>
      ))}
    </>
  );
};

interface ToastProviderProps {
  children: ComponentChildren;
}

export const ToastProvider = ({ children }: ToastProviderProps): JSX.Element => (
  <Toast.Provider toastManager={toastManager}>
    {children}
    <Toast.Portal>
      <Toast.Viewport className="fixed right-4 bottom-4 z-50 flex w-[calc(100vw-2rem)] flex-col gap-2 sm:w-96">
        <ToastList />
      </Toast.Viewport>
    </Toast.Portal>
  </Toast.Provider>
);
