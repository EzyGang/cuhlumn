import type { JSX } from "preact";
import { Link } from "wouter-preact";
import { LogOut } from "lucide-preact";
import { Button } from "../../shared/ui/Button.view";
import { Select } from "../../shared/ui/Select.view";
import type { AppShellProps } from "../../../types/appShell";
import { SiteNavContainer } from "../containers/SiteNav.container";

export const AppShellView = ({ data, status, actions, children }: AppShellProps): JSX.Element => (
  <div class="flex min-h-dvh flex-col">
    <header class="sticky top-0 z-30 border-b border-border bg-background">
      <div class="mx-auto flex max-w-7xl flex-wrap items-center gap-4 px-5 py-3">
        <Link href="/" class="text-sm font-semibold tracking-wide uppercase hover:opacity-80">
          Cuhlumn
        </Link>
        <Select
          ariaLabel="Current site"
          placeholder="Choose a site"
          value={data.host}
          items={data.hosts}
          onValueChange={actions.switchSite}
          class="w-64 max-w-full"
        />
        <Button
          variant="ghost"
          class="ml-auto"
          disabled={status.loggingOut}
          onClick={actions.logout}
        >
          <LogOut size={14} aria-hidden="true" />
          Log out
        </Button>
      </div>
      {data.host === null ? null : <SiteNavContainer host={data.host} />}
    </header>
    <main class="mx-auto flex w-full max-w-7xl flex-1 flex-col gap-6 px-5 py-8">{children}</main>
  </div>
);
