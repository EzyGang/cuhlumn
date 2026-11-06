import type { ComponentChildren, JSX } from "preact";
import { RotateCw } from "lucide-preact";
import type { LoadState } from "../../../types/load";
import { Button } from "./Button.view";

interface LoadGateProps {
  state: LoadState;
  loadingText: string;
  missingText: string;
  children: ComponentChildren;
}

export const LoadGate = ({
  state,
  loadingText,
  missingText,
  children,
}: LoadGateProps): JSX.Element => {
  if (state.status === "ready") {
    return <>{children}</>;
  }
  if (state.status === "loading") {
    return (
      <p class="text-sm text-muted" role="status">
        {loadingText}
      </p>
    );
  }
  return (
    <section class="panel flex flex-col items-start gap-4" role="alert">
      <p class="text-secondary">{state.status === "missing" ? missingText : state.error}</p>
      {state.status === "failed" ? (
        <Button onClick={state.retry}>
          <RotateCw size={14} aria-hidden="true" />
          Retry
        </Button>
      ) : null}
    </section>
  );
};
