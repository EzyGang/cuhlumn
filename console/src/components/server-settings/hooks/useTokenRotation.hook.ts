import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { rotateToken } from "../../../services/settings.service";
import type { RotationModel } from "../../../types/serverSettings";
import { reportApiError } from "../../../utils/reportApiError";
import { showToast } from "../../shared/ui/Toast.view";
import { useTokenDraft } from "../../setup/hooks/useTokenDraft.hook";

const rotate = async (token: string, rotating: Signal<boolean>): Promise<void> => {
  rotating.value = true;
  try {
    await rotateToken({ token });
    showToast(
      "success",
      "Admin token changed",
      "All other sessions ended. Store the new token now.",
    );
  } catch (error) {
    reportApiError(error, "Cannot change the token");
  } finally {
    rotating.value = false;
  }
};

export const useTokenRotation = (locked: boolean): RotationModel => {
  const token = useTokenDraft();
  const rotating = useSignal(false);
  return {
    data: { token, lock: locked ? "Set by CUHLUMN_TOKEN" : null },
    status: { rotating: rotating.value },
    actions: {
      rotate: (): void => void rotate(token.data.token, rotating),
    },
  };
};
