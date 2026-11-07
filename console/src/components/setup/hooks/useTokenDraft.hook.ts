import { useSignal } from "@preact/signals";
import type { TokenDraftModel } from "../../../types/auth";
import { generateToken, isTokenLongEnough } from "../../../utils/token";
import { showToast } from "../../shared/ui/Toast.view";

const copyToken = async (token: string): Promise<void> => {
  try {
    await navigator.clipboard.writeText(token);
    showToast("success", "Token copied", "Store it in a password manager.");
  } catch {
    showToast("error", "Cannot copy the token", "Select the token and copy it by hand.");
  }
};

/** Holds a new admin token with show, generate, and copy actions. */
export const useTokenDraft = (): TokenDraftModel => {
  const token = useSignal("");
  const visible = useSignal(false);
  return {
    data: {
      token: token.value,
      visible: visible.value,
      longEnough: isTokenLongEnough(token.value),
    },
    actions: {
      setToken: (value: string): void => {
        token.value = value;
      },
      toggleVisible: (): void => {
        visible.value = !visible.value;
      },
      generate: (): void => {
        token.value = generateToken();
        visible.value = true;
      },
      copy: (): void => void copyToken(token.value),
    },
  };
};
