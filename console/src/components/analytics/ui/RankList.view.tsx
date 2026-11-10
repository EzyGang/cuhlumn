import type { JSX } from "preact";
import type { RankListData } from "../../../types/analytics";

export const RankList = ({ title, emptyText, rows }: RankListData): JSX.Element => (
  <section class="panel flex flex-col gap-4">
    <h2 class="eyebrow">{title}</h2>
    {rows.length === 0 ? (
      <p class="text-sm text-muted">{emptyText}</p>
    ) : (
      <ul class="flex flex-col gap-3">
        {rows.map((row) => (
          <li key={row.key} class="flex flex-col gap-1.5">
            <div class="flex items-baseline justify-between gap-4 text-sm">
              <div class="flex min-w-0 flex-col">
                <span class="truncate text-white">{row.label}</span>
                {row.secondary === null ? null : (
                  <span class="truncate font-mono text-xs text-muted">{row.secondary}</span>
                )}
              </div>
              <span class="shrink-0 text-secondary tabular-nums">{row.count}</span>
            </div>
            <div class="h-1 bg-subtle" aria-hidden="true">
              <div class="h-full bg-muted" style={{ width: `${row.share}%` }} />
            </div>
          </li>
        ))}
      </ul>
    )}
  </section>
);
