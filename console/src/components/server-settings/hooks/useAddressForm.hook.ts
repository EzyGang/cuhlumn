import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import type { Loaded } from "../../../hooks/useLoad.hook";
import { saveSettings } from "../../../services/settings.service";
import type { Scheme } from "../../../types/api/Scheme";
import type { ServerSettings } from "../../../types/api/ServerSettings";
import type { AddressModel } from "../../../types/serverSettings";
import type { SelectItem } from "../../../types/select";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { reportFormError } from "../../../utils/reportApiError";
import { showToast } from "../../shared/ui/Toast.view";

const SCHEMES: readonly SelectItem<Scheme>[] = [
  { value: "https", label: "HTTPS" },
  { value: "http", label: "HTTP" },
];

interface AddressState {
  host: Signal<string | null>;
  scheme: Signal<Scheme | null>;
  saving: Signal<boolean>;
  errors: Signal<FieldErrors>;
  movedTo: Signal<string | null>;
}

/** Returns the console address on a new host, or null when the console stays on this host. */
const newConsoleAddress = (before: ServerSettings, after: ServerSettings): string | null => {
  const host = after.console_host;
  if (host === null || host === before.console_host || host === window.location.hostname) {
    return null;
  }
  const port = window.location.port === "" ? "" : `:${window.location.port}`;
  return `${after.scheme}://${host}${port}/`;
};

const save = async (settings: Loaded<ServerSettings>, state: AddressState): Promise<void> => {
  const before = settings.value;
  if (before === null) {
    return;
  }
  const host = (state.host.value ?? before.console_host ?? "").trim();
  state.saving.value = true;
  try {
    const after = await saveSettings({
      console_host: host === "" ? null : host,
      scheme: state.scheme.value ?? before.scheme,
    });
    settings.set(after);
    state.host.value = null;
    state.scheme.value = null;
    state.errors.value = {};
    state.movedTo.value = newConsoleAddress(before, after);
    showToast("success", "Server settings saved");
  } catch (error) {
    state.errors.value = reportFormError(error, ["console_host"], "Cannot save the settings");
  } finally {
    state.saving.value = false;
  }
};

const lockText = (locked: boolean, variable: string): string | null =>
  locked ? `Set by ${variable}` : null;

const addressData = (
  state: AddressState,
  current: ServerSettings | null,
): AddressModel["data"] => ({
  host: state.host.value ?? current?.console_host ?? "",
  scheme: state.scheme.value ?? current?.scheme ?? "https",
  schemes: SCHEMES,
  hostLock: lockText(current?.locked.console_host === true, "CUHLUMN_CONSOLE_HOST"),
  schemeLock: lockText(current?.locked.scheme === true, "CUHLUMN_SCHEME"),
  hostError: state.errors.value["console_host"],
  movedTo: state.movedTo.value,
});

export const useAddressForm = (settings: Loaded<ServerSettings>): AddressModel => {
  const state: AddressState = {
    host: useSignal(null),
    scheme: useSignal(null),
    saving: useSignal(false),
    errors: useSignal({}),
    movedTo: useSignal(null),
  };
  return {
    data: addressData(state, settings.value),
    status: {
      saving: state.saving.value,
      dirty: state.host.value !== null || state.scheme.value !== null,
    },
    actions: {
      setHost: (value: string): void => {
        state.host.value = value;
      },
      setScheme: (value: Scheme): void => {
        state.scheme.value = value;
      },
      save: (event: Event): void => {
        event.preventDefault();
        void save(settings, state);
      },
    },
  };
};
