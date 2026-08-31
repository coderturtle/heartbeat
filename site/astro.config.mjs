// @ts-check
import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";
import tailwind from "@astrojs/tailwind";

// https://astro.build/config
export default defineConfig({
  // Custom domain (heartbeat.coderturtle.io) via GitHub Pages + Route53 CNAME,
  // see agentic-infra-lab's patterns/github-pages-dns (onboarded as the sixth
  // consumer, DNS not yet live as of this cutover - see docs/next-actions.md).
  // Site serves at the domain root, not under /heartbeat/ on
  // coderturtle.github.io, once DNS/verification/Pages enablement are all
  // done. Every internal link MUST still be base-aware
  // (import.meta.env.BASE_URL), not a bare "/path".
  site: "https://heartbeat.coderturtle.io",
  base: "/",
  integrations: [mdx(), tailwind()],
  output: "static",
});
