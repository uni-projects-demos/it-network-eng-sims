import adapter from "@sveltejs/adapter-static";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

const base = (process.env.BASE_PATH ?? "") as "" | `/${string}`;

export default defineConfig({
  server: { allowedHosts: ["terminal.local"] },
  plugins: [sveltekit({ adapter: adapter(), paths: { base } })],
});
