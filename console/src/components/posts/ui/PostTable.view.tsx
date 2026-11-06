import type { JSX } from "preact";
import type { PostRow } from "../../../types/posts";
import { PostRowView } from "./PostRow.view";

interface PostTableProps {
  rows: PostRow[];
}

const COLUMNS: readonly string[] = ["Title", "Status", "Kind", "Updated"];

export const PostTableView = ({ rows }: PostTableProps): JSX.Element => (
  <div class="overflow-x-auto rounded-sm border border-border">
    <table class="w-full min-w-[40rem] border-collapse text-sm">
      <thead class="bg-surface text-left">
        <tr>
          {COLUMNS.map((column) => (
            <th key={column} scope="col" class="eyebrow px-4 py-3">
              {column}
            </th>
          ))}
          <th scope="col" class="eyebrow px-4 py-3">
            <span class="sr-only">Public page</span>
          </th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <PostRowView key={row.slug} row={row} />
        ))}
      </tbody>
    </table>
  </div>
);
