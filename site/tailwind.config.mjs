/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{astro,html,js,jsx,md,mdx,ts,tsx}"],
  theme: {
    extend: {
      // Brand layers extend these tokens per series. Keep the base neutral.
      // RGB-channel custom properties (see global.css), so bg-ink/5-style opacity
      // modifiers keep working, and the same utility classes cover both light and
      // dark automatically via prefers-color-scheme.
      colors: {
        ink: "rgb(var(--ink-rgb) / <alpha-value>)",
        paper: "rgb(var(--paper-rgb) / <alpha-value>)",
        accent: "rgb(var(--accent-rgb) / <alpha-value>)",
      },
    },
  },
  plugins: [],
};
