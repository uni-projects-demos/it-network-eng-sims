<script lang="ts">
import { onMount } from "svelte";
import { type Locale, locale, translate } from "#lib/i18n/index.ts";

let curLocale: Locale;
let isDark: boolean = true;

$: curLocale = $locale;

function applyTheme(isNxt: boolean, persist: boolean = true): void {
  isDark = isNxt;

  const root: HTMLElement = document.documentElement;
  const theme: "dark" | "light" = isDark ? "dark" : "light";

  root.dataset.theme = theme;
  root.style.colorScheme = theme;

  const meta: HTMLMetaElement | null = document.querySelector<HTMLMetaElement>(
    'meta[name="theme-color"]',
  );
  const themeColor: string | undefined = isDark ? meta?.dataset.dark : meta?.dataset.light;
  if (meta !== null && themeColor !== undefined) {
    meta.content = themeColor;
  }

  if (persist) {
    try {
      localStorage.setItem("network-sim-theme", theme);
    } catch {}
  }
}

onMount((): void => {
  let initial: boolean = true;

  try {
    const saved: string | null = localStorage.getItem("network-sim-theme");
    if (saved === "light") {
      initial = false;
    } else if (saved === "dark") {
      initial = true;
    } else {
      const mediaQuery: MediaQueryList = window.matchMedia("(prefers-color-scheme: dark)");
      initial = mediaQuery.matches;
    }
  } catch {
    initial = true;
  }

  applyTheme(initial, false);
});
</script>

<button
  class="icon-button w-11"
  type="button"
  aria-label={translate(isDark ? 'nav.lightMode' : 'nav.darkMode', curLocale)}
  title={translate(isDark ? 'nav.lightMode' : 'nav.darkMode', curLocale)}
  aria-pressed={isDark}
  onclick={() => applyTheme(!isDark)}
>
  {#if isDark}
    <svg
      class="size-[21px] fill-none stroke-current stroke-[1.8] [stroke-linecap:round] [stroke-linejoin:round]"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
      <circle cx="12" cy="12" r="4"></circle>
      <path d="M12 2v2M12 20v2M4.93 4.93l1.42 1.42M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.42-1.42M17.66 6.34l1.41-1.41"></path>
    </svg>
  {:else}
    <svg
      class="size-[21px] fill-none stroke-current stroke-[1.8] [stroke-linecap:round] [stroke-linejoin:round]"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
      <path d="M20.5 13A8.5 8.5 0 0 1 11 3.5 8.5 8.5 0 1 0 20.5 13Z"></path>
    </svg>
  {/if}
</button>
