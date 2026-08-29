// @ts-check
import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";
import tailwind from "@astrojs/tailwind";

// https://astro.build/config
export default defineConfig({
  // GitHub Pages project hosting: coderturtle.github.io/heartbeat/. No custom
  // domain yet (that's a later, human-gated decision, same as borrow-native's
  // own CNAME PR came after its initial scaffold, not with it) - so base is a
  // real path prefix here, not "/". Every internal link MUST be base-aware
  // (import.meta.env.BASE_URL), not a bare "/path", or Pages project hosting
  // breaks silently.
  site: "https://coderturtle.github.io",
  base: "/heartbeat/",
  integrations: [mdx(), tailwind()],
  output: "static",
});
