import type { JSX } from "preact";
import { Redirect, Route, Switch } from "wouter-preact";
import { useSessionCheck } from "../hooks/useSessionCheck.hook";
import { LoginPage } from "../pages/Login.page";
import { sessionStore } from "../stores/session.store";
import { SiteRoutes } from "./SiteRoutes";

const LoginRoute = (): JSX.Element =>
  sessionStore.status.value === "authenticated" ? <Redirect to="/" replace /> : <LoginPage />;

const ProtectedRoute = (): JSX.Element => {
  const status = sessionStore.status.value;
  if (status === "anonymous") {
    return <Redirect to="/login" replace />;
  }
  return status === "checking" ? (
    <p class="p-8 text-sm text-muted" role="status">
      Checking the session
    </p>
  ) : (
    <SiteRoutes />
  );
};

export const AppRoutes = (): JSX.Element => {
  useSessionCheck();
  return (
    <Switch>
      <Route path="/login">
        {sessionStore.status.value === "checking" ? null : <LoginRoute />}
      </Route>
      <Route>
        <ProtectedRoute />
      </Route>
    </Switch>
  );
};
