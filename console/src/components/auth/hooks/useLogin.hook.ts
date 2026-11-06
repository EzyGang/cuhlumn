import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { useLocation } from "wouter-preact";
import { createSession } from "../../../services/session.service";
import { sessionStore } from "../../../stores/session.store";
import type { LoginModel } from "../../../types/auth";
import type { Navigate } from "../../../types/navigation";
import { errorMessage, isUnauthorized } from "../../../utils/reportApiError";

interface LoginState {
  token: Signal<string>;
  error: Signal<string | null>;
  submitting: Signal<boolean>;
}

const logIn = async (
  { token, error, submitting }: LoginState,
  navigate: Navigate,
): Promise<void> => {
  submitting.value = true;
  error.value = null;
  try {
    await createSession({ token: token.value });
    sessionStore.status.value = "authenticated";
    navigate("/", { replace: true });
  } catch (failure) {
    error.value = isUnauthorized(failure) ? "The token is not correct." : errorMessage(failure);
  } finally {
    submitting.value = false;
  }
};

export const useLogin = (): LoginModel => {
  const [, navigate] = useLocation();
  const state: LoginState = {
    token: useSignal(""),
    error: useSignal<string | null>(null),
    submitting: useSignal(false),
  };
  return {
    data: { token: state.token.value, error: state.error.value },
    status: { submitting: state.submitting.value },
    actions: {
      setToken: (value: string): void => {
        state.token.value = value;
      },
      submit: (event: Event): void => {
        event.preventDefault();
        void logIn(state, navigate);
      },
    },
  };
};
