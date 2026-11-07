import type { JSX } from "preact";
import { Section } from "../../../shared/ui/Section.view";
import { AppearanceGroup } from "./AppearanceGroup.view";
import { CodeGroup } from "./CodeGroup.view";
import { ColourGroup } from "./ColourGroup.view";
import { LayoutGroup } from "./LayoutGroup.view";
import { PageExtras } from "./PageExtras.view";
import { ShareImageGroup } from "./ShareImageGroup.view";
import { SocialGroup } from "./SocialGroup.view";

export const AppearanceSection = (): JSX.Element => (
  <Section title="Appearance" description="Colours, layout, profiles, and share images.">
    <ColourGroup />
    <LayoutGroup />
    <CodeGroup />
    <SocialGroup />
    <ShareImageGroup />
    <AppearanceGroup title="Page extras">
      <PageExtras />
    </AppearanceGroup>
  </Section>
);
