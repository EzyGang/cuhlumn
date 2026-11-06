import type { JSX } from "preact";
import { Plus } from "lucide-preact";
import { LinkButton } from "../../shared/ui/LinkButton.view";
import { LoadGate } from "../../shared/ui/LoadGate.view";
import { PageHeader } from "../../shared/ui/PageHeader.view";
import type { PostsModel } from "../../../types/posts";
import { PostFiltersView } from "./PostFilters.view";
import { PostTableView } from "./PostTable.view";

export const PostsView = ({ data, status, actions }: PostsModel): JSX.Element => (
  <section class="flex flex-col gap-6">
    <PageHeader eyebrow="Posts" title="Posts and pages">
      <LinkButton href={data.newHref} variant="primary">
        <Plus size={14} aria-hidden="true" />
        New post
      </LinkButton>
    </PageHeader>
    <LoadGate
      state={status.load}
      loadingText="Loading the posts"
      missingText="This site does not exist."
    >
      <PostFiltersView data={data} actions={actions} />
      {data.rows.length === 0 ? (
        <p class="panel text-sm text-secondary">{data.emptyMessage}</p>
      ) : (
        <PostTableView rows={data.rows} />
      )}
    </LoadGate>
  </section>
);
