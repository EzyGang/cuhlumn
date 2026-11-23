import type { ComponentChildren, JSX } from "preact";
import { Redirect, useLocation, useRoute } from "wouter-preact";
import { sitesStore } from "../stores/sites.store";
import { analyticsPath, siteKind } from "../utils/sites";

interface ExternalSiteGuardProps {
  children: ComponentChildren;
}

/**
 * Sends every page of an external site to its analytics.
 * An external site has no posts, images, or settings, so the other site pages are for blogs only.
 */
export const ExternalSiteGuard = ({ children }: ExternalSiteGuardProps): JSX.Element => {
  const [location] = useLocation();
  const [, params] = useRoute<{ host: string }>("/sites/:host/*?");
  const host = params?.host ?? null;
  if (host === null || siteKind(sitesStore.sites.value, host) !== "external") {
    return <>{children}</>;
  }
  const target = analyticsPath(host);
  return location.replace(/\/+$/, "") === target ? (
    <>{children}</>
  ) : (
    <Redirect to={target} replace />
  );
};
