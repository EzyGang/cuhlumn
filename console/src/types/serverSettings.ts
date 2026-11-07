import type { Scheme } from "./api/Scheme";
import type { TokenDraftModel } from "./auth";
import type { LoadState } from "./load";
import type { SelectItem } from "./select";

export interface AddressModel {
  data: {
    host: string;
    scheme: Scheme;
    schemes: readonly SelectItem<Scheme>[];
    hostLock: string | null;
    schemeLock: string | null;
    hostError: string | undefined;
    movedTo: string | null;
  };
  status: { saving: boolean; dirty: boolean };
  actions: {
    setHost: (host: string) => void;
    setScheme: (scheme: Scheme) => void;
    save: (event: Event) => void;
  };
}

export interface RotationModel {
  data: { token: TokenDraftModel; lock: string | null };
  status: { rotating: boolean };
  actions: { rotate: () => void };
}

export interface ServerSettingsModel {
  data: { address: AddressModel; rotation: RotationModel };
  status: { load: LoadState };
}
