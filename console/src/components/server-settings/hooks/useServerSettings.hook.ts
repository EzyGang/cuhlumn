import { useLoad } from "../../../hooks/useLoad.hook";
import { getSettings } from "../../../services/settings.service";
import type { ServerSettingsModel } from "../../../types/serverSettings";
import { useAddressForm } from "./useAddressForm.hook";
import { useTokenRotation } from "./useTokenRotation.hook";

export const useServerSettings = (): ServerSettingsModel => {
  const settings = useLoad("settings", getSettings);
  const address = useAddressForm(settings);
  const rotation = useTokenRotation(settings.value?.locked.token === true);
  return {
    data: { address, rotation },
    status: { load: { status: settings.status, error: settings.error, retry: settings.retry } },
  };
};
