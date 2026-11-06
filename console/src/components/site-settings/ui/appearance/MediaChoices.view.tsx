import type { JSX } from "preact";
import type { MediaChoice } from "../../../../types/siteSettings";
import { MediaChoiceView } from "./MediaChoice.view";

interface MediaChoicesProps {
  items: MediaChoice[];
  loading: boolean;
  error: string | null;
}

export const MediaChoicesView = ({ items, loading, error }: MediaChoicesProps): JSX.Element => {
  if (loading || error !== null) {
    return (
      <p class="text-sm text-muted" role="status">
        {error ?? "Loading the media library"}
      </p>
    );
  }
  if (items.length === 0) {
    return (
      <p class="text-sm text-secondary">
        There are no images yet. Upload images on the Media screen.
      </p>
    );
  }
  return (
    <ul class="grid grid-cols-2 gap-3 sm:grid-cols-4">
      {items.map((item) => (
        <MediaChoiceView key={item.name} item={item} />
      ))}
    </ul>
  );
};
