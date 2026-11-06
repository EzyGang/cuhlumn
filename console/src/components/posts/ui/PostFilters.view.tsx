import type { JSX } from "preact";
import { Input } from "../../shared/ui/Input.view";
import { Select } from "../../shared/ui/Select.view";
import type { PostsModel } from "../../../types/posts";

type PostFiltersProps = Pick<PostsModel, "data" | "actions">;

export const PostFiltersView = ({ data, actions }: PostFiltersProps): JSX.Element => (
  <div class="flex flex-wrap items-center gap-3">
    <Input
      type="search"
      label="Search by title or slug"
      placeholder="Search by title or slug"
      value={data.filters.query}
      onValueChange={actions.setQuery}
      class="max-w-xs"
    />
    <Select
      ariaLabel="Status filter"
      value={data.filters.status}
      items={data.statusItems}
      onValueChange={actions.setStatus}
    />
    <Select
      ariaLabel="Kind filter"
      value={data.filters.kind}
      items={data.kindItems}
      onValueChange={actions.setKind}
    />
    <p class="ml-auto text-sm text-muted" role="status">
      {data.countText}
    </p>
  </div>
);
