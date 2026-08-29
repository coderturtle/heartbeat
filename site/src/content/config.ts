import { defineCollection, z } from "astro:content";
import { glob } from "astro/loaders";

// The build-log lives in docs/build-log/ at the repo root, not under src/. This is
// the maintainer's deliberate, dated journal (see docs/workshop-design.md and
// the Workshop Gremlin's Risks note: entries are written, not machine-generated from
// session logs). The glob loader reads it in place; nothing is duplicated into src/.
const buildlog = defineCollection({
  loader: glob({ pattern: "*.md", base: new URL("../../../docs/build-log", import.meta.url) }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    pubDate: z.coerce.date(),
    tags: z.array(z.string()).default([]),
    draft: z.boolean().default(false),
  }),
});

export const collections = { buildlog };
