import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import type { RefObject } from "preact";
import { useRef } from "preact/hooks";
import type { TokenDraftModel } from "../../../types/auth";
import { writeClipboard } from "../../../utils/clipboard";
import { generateToken, isTokenLongEnough } from "../../../utils/token";
import { showToast } from "../../shared/ui/Toast.view";

/** Shows the token and selects it in the input after the next render, so the admin can copy it by hand. */
const selectToken = (input: RefObject<HTMLInputElement>, visible: Signal<boolean>): void => {
  visible.value = true;
  requestAnimationFrame(() => {
    input.current?.focus();
    input.current?.select();
  });
};

const copyToken = async (
  token: string,
  input: RefObject<HTMLInputElement>,
  visible: Signal<boolean>,
): Promise<void> => {
  if (await writeClipboard(token)) {
    showToast("success", "Token copied", "Store it in a password manager.");
    return;
  }
  selectToken(input, visible);
  showToast("error", "Cannot copy the token", "Select the token and copy it by hand.");
};

/** Holds a new admin token with show, generate, and copy actions. */
export const useTokenDraft = (): TokenDraftModel => {
  const token = useSignal("");
  const visible = useSignal(false);
  const input = useRef<HTMLInputElement>(null);
  return {
    data: {
      token: token.value,
      visible: visible.value,
      longEnough: isTokenLongEnough(token.value),
      input,
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
      copy: (): void => void copyToken(token.value, input, visible),
    },
  };
};
