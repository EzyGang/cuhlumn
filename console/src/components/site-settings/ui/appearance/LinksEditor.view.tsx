import type { JSX } from "preact";
import { Plus } from "lucide-preact";
import { Button } from "../../../shared/ui/Button.view";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";
import { LinkRowView } from "./LinkRow.view";

export const LinksEditor = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <div class="flex flex-col gap-3" role="group" aria-labelledby="header-links-title">
      <p id="header-links-title" class="text-xs font-medium tracking-wide text-secondary uppercase">
        {data.linksTitle}
      </p>
      {data.errors["brand.links"] === undefined ? null : (
        <p class="text-xs text-error" role="alert">
          {data.errors["brand.links"]}
        </p>
      )}
      {data.links.map((link) => (
        <LinkRowView key={link.id} link={link} />
      ))}
      <div>
        <Button onClick={actions.addLink} disabled={!data.canAddLink}>
          <Plus size={14} aria-hidden="true" />
          Add link
        </Button>
      </div>
    </div>
  );
};
