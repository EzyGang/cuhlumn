import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { useLocation } from "wouter-preact";
import { ApiError } from "../../../services/http";
import { setUp } from "../../../services/session.service";
import { sessionStore } from "../../../stores/session.store";
import type { SetupModel } from "../../../types/auth";
import type { Navigate } from "../../../types/navigation";
import { errorMessage } from "../../../utils/reportApiError";
import { showToast } from "../../shared/ui/Toast.view";
import { useTokenDraft } from "./useTokenDraft.hook";

interface SetupState {
  error: Signal<string | null>;
  submitting: Signal<boolean>;
}

const isConflict = (failure: unknown): boolean =>
  failure instanceof ApiError && failure.body.error.code === "conflict";

const submitSetup = async (token: string, state: SetupState, navigate: Navigate): Promise<void> => {
  state.submitting.value = true;
  state.error.value = null;
  try {
    await setUp({ token });
    sessionStore.status.value = "authenticated";
    navigate("/", { replace: true });
  } catch (failure) {
    if (isConflict(failure)) {
      showToast("error", "The server has a token already", "Log in with that token.");
      sessionStore.status.value = "anonymous";
    } else {
      state.error.value = errorMessage(failure);
    }
  } finally {
    state.submitting.value = false;
  }
};

export const useSetup = (): SetupModel => {
  const [, navigate] = useLocation();
  const token = useTokenDraft();
  const state: SetupState = { error: useSignal(null), submitting: useSignal(false) };
  return {
    data: { token, error: state.error.value },
    status: { submitting: state.submitting.value },
    actions: {
      submit: (event: Event): void => {
        event.preventDefault();
        void submitSetup(token.data.token, state, navigate);
      },
    },
  };
};
