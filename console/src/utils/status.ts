import type { BadgeTone } from "../components/shared/ui/Badge.view";
import type { Status } from "../types/api/Status";

const STATUS_TONES: Record<Status, BadgeTone> = { draft: "warning", published: "success" };

export const statusTone = (status: Status): BadgeTone => STATUS_TONES[status];
