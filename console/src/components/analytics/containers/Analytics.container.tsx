import type { JSX } from "preact";
import { useAnalytics } from "../hooks/useAnalytics.hook";
import { AnalyticsView } from "../ui/Analytics.view";

export const AnalyticsContainer = (): JSX.Element => <AnalyticsView {...useAnalytics()} />;
