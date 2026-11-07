export interface LoginModel {
  data: { token: string; error: string | null };
  status: { submitting: boolean };
  actions: { setToken: (token: string) => void; submit: (event: Event) => void };
}

/** A new admin token that the admin types or generates. */
export interface TokenDraftModel {
  data: { token: string; visible: boolean; longEnough: boolean };
  actions: {
    setToken: (token: string) => void;
    toggleVisible: () => void;
    generate: () => void;
    copy: () => void;
  };
}

export interface SetupModel {
  data: { token: TokenDraftModel; error: string | null };
  status: { submitting: boolean };
  actions: { submit: (event: Event) => void };
}
