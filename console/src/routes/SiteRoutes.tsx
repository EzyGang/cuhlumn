import type { JSX } from "preact";
import { Redirect, Route, Switch } from "wouter-preact";
import { AppShellContainer } from "../components/app-shell/containers/AppShell.container";
import { AnalyticsPage } from "../pages/Analytics.page";
import { FontsPage } from "../pages/Fonts.page";
import { MediaPage } from "../pages/Media.page";
import { PostEditorPage } from "../pages/PostEditor.page";
import { PostsPage } from "../pages/Posts.page";
import { ServerSettingsPage } from "../pages/ServerSettings.page";
import { SiteSettingsPage } from "../pages/SiteSettings.page";
import { SitesPage } from "../pages/Sites.page";

export const SiteRoutes = (): JSX.Element => (
  <AppShellContainer>
    <Switch>
      <Route path="/" component={SitesPage} />
      <Route path="/settings" component={ServerSettingsPage} />
      <Route path="/fonts" component={FontsPage} />
      <Route path="/new-site">
        <SiteSettingsPage key="create-site" />
      </Route>
      <Route path="/sites/:host">
        {(params) => <SiteSettingsPage key={`site:${params.host}`} />}
      </Route>
      <Route path="/sites/:host/posts">{(params) => <PostsPage key={params.host} />}</Route>
      <Route path="/sites/:host/new-post">
        {(params) => <PostEditorPage key={`create:${params.host}`} />}
      </Route>
      <Route path="/sites/:host/posts/:slug">
        {(params) => <PostEditorPage key={`edit:${params.host}/${params.slug}`} />}
      </Route>
      <Route path="/sites/:host/media">{(params) => <MediaPage key={params.host} />}</Route>
      <Route path="/sites/:host/analytics">{(params) => <AnalyticsPage key={params.host} />}</Route>
      <Route>
        <Redirect to="/" replace />
      </Route>
    </Switch>
  </AppShellContainer>
);
