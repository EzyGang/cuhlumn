import type { JSX } from "preact";
import { LoadGate } from "../../shared/ui/LoadGate.view";
import { PageHeader } from "../../shared/ui/PageHeader.view";
import { Select } from "../../shared/ui/Select.view";
import type { AnalyticsModel } from "../../../types/analytics";
import { DailyChart } from "./DailyChart.view";
import { RankList } from "./RankList.view";

type ReportProps = Pick<AnalyticsModel, "data">;

const Totals = ({ data }: ReportProps): JSX.Element => (
  <section class="grid gap-4 sm:grid-cols-2">
    <div class="panel flex flex-col gap-2">
      <h2 class="eyebrow">Views</h2>
      <p class="text-3xl font-semibold tabular-nums">{data.views}</p>
    </div>
    <div class="panel flex flex-col gap-2">
      <h2 class="eyebrow">Visitors</h2>
      <p class="text-3xl font-semibold tabular-nums">{data.visitors}</p>
      <p class="text-xs text-muted">
        This is the sum of the daily unique visitors. A reader who comes back on a different day
        counts again.
      </p>
    </div>
  </section>
);

const Report = ({ data }: ReportProps): JSX.Element =>
  data.empty ? (
    <p class="panel text-sm text-secondary">
      No views yet. Views appear here when readers open the blog.
    </p>
  ) : (
    <>
      <Totals data={data} />
      <DailyChart chart={data.chart} />
      <div class="grid gap-4 lg:grid-cols-2">
        {data.lists.map((list) => (
          <RankList key={list.title} {...list} />
        ))}
      </div>
    </>
  );

export const AnalyticsView = ({ data, status, actions }: AnalyticsModel): JSX.Element => (
  <section class="flex flex-col gap-6">
    <PageHeader
      eyebrow="Analytics"
      title="Views and visitors"
      below={<p class="text-sm text-secondary">{data.rangeText}</p>}
    >
      <Select
        ariaLabel="Period"
        value={data.period}
        items={data.periodItems}
        onValueChange={actions.setPeriod}
      />
    </PageHeader>
    <LoadGate
      state={status.load}
      loadingText="Loading the analytics"
      missingText="This site does not exist."
    >
      <Report data={data} />
    </LoadGate>
  </section>
);
