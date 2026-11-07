import type { JSX } from "preact";
import { Plus } from "lucide-preact";
import { Button } from "../../../shared/ui/Button.view";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";
import { SocialRowView } from "./SocialRow.view";

export const SocialEditor = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <div class="flex flex-col gap-3" role="group" aria-labelledby="social-links-title">
      <p id="social-links-title" class="text-xs font-medium tracking-wide text-secondary uppercase">
        {data.social.title}
      </p>
      {data.errors["brand.social"] === undefined ? null : (
        <p class="text-xs text-error" role="alert">
          {data.errors["brand.social"]}
        </p>
      )}
      {data.social.rows.map((row) => (
        <SocialRowView key={row.id} row={row} networks={data.social.networks} />
      ))}
      <div>
        <Button onClick={actions.addSocial} disabled={!data.social.canAdd}>
          <Plus size={14} aria-hidden="true" />
          Add profile
        </Button>
      </div>
    </div>
  );
};
