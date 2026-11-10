import type { JSX } from "preact";
import type { DailyChartData } from "../../../types/analytics";

interface DailyChartProps {
  chart: DailyChartData;
}

const ChartAxes = ({ chart }: DailyChartProps): JSX.Element => (
  <g class="fill-muted text-[11px]">
    {chart.ticks.map((tick) => (
      <g key={tick.key}>
        <line
          x1={chart.plot.left}
          x2={chart.plot.right}
          y1={tick.position}
          y2={tick.position}
          class="stroke-border"
        />
        <text
          x={chart.plot.left - 8}
          y={tick.position}
          text-anchor="end"
          dominant-baseline="middle"
        >
          {tick.text}
        </text>
      </g>
    ))}
    {chart.days.map((day) => (
      <text key={day.key} x={day.position} y={chart.height - 8} text-anchor="middle">
        {day.text}
      </text>
    ))}
  </g>
);

const ChartBars = ({ chart }: DailyChartProps): JSX.Element => (
  <g>
    {chart.bars.map((bar) => (
      <g key={bar.key} class="group">
        <title>{bar.title}</title>
        <rect
          x={bar.slotX}
          y={chart.plot.top}
          width={bar.slotWidth}
          height={chart.plot.bottom - chart.plot.top}
          class="fill-transparent group-hover:fill-hover"
        />
        <rect x={bar.x} y={bar.y} width={bar.width} height={bar.height} class="fill-muted" />
      </g>
    ))}
  </g>
);

const ChartTable = ({ chart }: DailyChartProps): JSX.Element => (
  <table class="sr-only">
    <caption>Views and visitors per day</caption>
    <thead>
      <tr>
        <th scope="col">Day</th>
        <th scope="col">Views</th>
        <th scope="col">Visitors</th>
      </tr>
    </thead>
    <tbody>
      {chart.rows.map((row) => (
        <tr key={row.key}>
          <th scope="row">{row.date}</th>
          <td>{row.views}</td>
          <td>{row.visitors}</td>
        </tr>
      ))}
    </tbody>
  </table>
);

const ChartLegend = (): JSX.Element => (
  <div class="flex flex-wrap gap-4 text-xs text-muted" aria-hidden="true">
    <span class="flex items-center gap-2">
      <span class="size-2.5 bg-muted" />
      Views
    </span>
    <span class="flex items-center gap-2">
      <span class="h-0.5 w-3 bg-emerald" />
      Visitors
    </span>
  </div>
);

export const DailyChart = ({ chart }: DailyChartProps): JSX.Element => (
  <section class="panel flex flex-col gap-4">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <h2 class="eyebrow">Views per day</h2>
      <ChartLegend />
    </div>
    <div class="overflow-x-auto">
      <svg
        viewBox={`0 0 ${chart.width} ${chart.height}`}
        class="h-auto w-full min-w-[720px]"
        aria-hidden="true"
      >
        <ChartAxes chart={chart} />
        <ChartBars chart={chart} />
        <polyline
          points={chart.visitorsLine}
          fill="none"
          stroke-width="2"
          stroke-linejoin="round"
          class="pointer-events-none stroke-emerald"
        />
      </svg>
    </div>
    <ChartTable chart={chart} />
  </section>
);
