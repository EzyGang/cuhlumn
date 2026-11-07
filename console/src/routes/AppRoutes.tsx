import type { JSX } from "preact";
import { Redirect, Route, Switch, useLocation } from "wouter-preact";
import { useSessionCheck } from "../hooks/useSessionCheck.hook";
import { LoginPage } from "../pages/Login.page";
import { SetupPage } from "../pages/Setup.page";
import { sessionStore } from "../stores/session.store";
import { redirectTarget } from "../utils/setup";
import { SiteRoutes } from "./SiteRoutes";

export const AppRoutes = (): JSX.Element => {
  useSessionCheck();
  const [location] = useLocation();
  const status = sessionStore.status.value;
  const target = redirectTarget(status, location.replace(/\/+$/, "") || "/");
  if (target !== null) {
    return <Redirect to={target} replace />;
  }
  if (status === "checking") {
    return (
      <p class="p-8 text-sm text-muted" role="status">
        Checking the session
      </p>
    );
  }
  return (
    <Switch>
      <Route path="/login" component={LoginPage} />
      <Route path="/setup" component={SetupPage} />
      <Route>
        <SiteRoutes />
      </Route>
    </Switch>
  );
};
