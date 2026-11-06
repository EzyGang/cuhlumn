import type { JSX } from "preact";
import { Section } from "../../../shared/ui/Section.view";
import { AccentFields } from "./AccentFields.view";
import { ImageFields } from "./ImageFields.view";
import { LinksEditor } from "./LinksEditor.view";
import { PageExtras } from "./PageExtras.view";

export const AppearanceSection = (): JSX.Element => (
  <Section title="Appearance" description="Colors, type, images, and extra page content.">
    <AccentFields />
    <ImageFields />
    <LinksEditor />
    <PageExtras />
  </Section>
);
