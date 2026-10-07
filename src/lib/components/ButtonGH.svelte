<script lang="ts">
import { locale, translate } from "#lib/i18n/index.ts";

export const href: string = "https://github.com/r055a/it-network-eng-sims";
</script>

<a
  class="icon-button w-11 no-underline"
  href={href}
  target="_blank"
  rel="noopener noreferrer"
  aria-label={translate('nav.ghRepo', $locale)}
  title={translate('nav.ghRepo', $locale)}
>
  <svg
    class="size-5 fill-current"
    viewBox="0 0 16 16"
    aria-hidden="true"
  >
    <path
      d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38v-1.49c-2.23.48-2.7-1.08-2.7-1.08-.36-.92-.89-1.17-.89-1.17-.73-.5.06-.49.06-.49.8.06 1.22.82 1.22.82.71 1.22 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82A7.65 7.65 0 0 1 8 3.72c.68 0 1.36.09 2 .27 1.53-1.03 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.28.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.74.54 1.48v2.2c0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z"
    ></path>
  </svg>
</a>
