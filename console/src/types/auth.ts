export interface LoginModel {
  data: { token: string; error: string | null };
  status: { submitting: boolean };
  actions: { setToken: (token: string) => void; submit: (event: Event) => void };
}
