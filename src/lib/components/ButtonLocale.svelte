<script lang="ts">
import { onMount } from "svelte";
import {
  type Locale,
  locale,
  localeDisplayName,
  setLocale,
  supportedLocales,
  translate,
} from "#lib/i18n/index.ts";

let dialog: HTMLDialogElement | undefined;
let search: string = "";

let cur: Locale;
let displayNames: Record<Locale, string>;
let query: string;
let filtered: Locale[];

$: cur = $locale;

$: displayNames = Object.fromEntries(
  supportedLocales.map((code: Locale): [Locale, string] => [code, localeDisplayName(code, cur)]),
) as Record<Locale, string>;

$: query = search.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase().trim();

$: filtered = supportedLocales.filter((code: Locale): boolean =>
  `${displayNames[code]} ${code}`
    .normalize("NFD")
    .replace(/\p{M}/gu, "")
    .toLowerCase()
    .includes(query),
);

onMount((): void => {
  try {
    const saved: string | null = localStorage.getItem("network-sim-locale");
    if (saved !== null && supportedLocales.includes(saved)) {
      void setLocale(saved);
    }
  } catch {}
});

function openDialog(): void {
  search = "";
  dialog?.showModal();
}

function closeDialog(): void {
  dialog?.close();
}

function handleDialogClick(event: MouseEvent): void {
  if (!dialog || event.target !== dialog) {
    return;
  }

  const box: DOMRect = dialog.getBoundingClientRect();

  const outside: boolean =
    event.clientX < box.left ||
    event.clientX > box.right ||
    event.clientY < box.top ||
    event.clientY > box.bottom;
  if (outside) {
    dialog.close();
  }
}

async function choose(code: Locale): Promise<void> {
  await setLocale(code);
  dialog?.close();
}
</script>

<button
  class="icon-button gap-1.5 whitespace-nowrap"
  type="button"
  aria-label={translate('nav.chooseLanguage', cur)}
  aria-haspopup="dialog"
  onclick={openDialog}
>
  <svg
    class="size-5 fill-none stroke-current stroke-[1.8] [stroke-linecap:round] [stroke-linejoin:round]"
    viewBox="0 0 24 24"
    aria-hidden="true"
  >
    <circle cx="12" cy="12" r="9"></circle>
    <path d="M3 12h18M12 3a14.5 14.5 0 0 1 0 18M12 3a14.5 14.5 0 0 0 0 18"></path>
  </svg>
  <span class="text-[11px] leading-none font-extrabold tracking-[.035em]" translate="no">{cur.toUpperCase()}</span>
</button>

<dialog
  bind:this={dialog}
  class="locale-dialog m-auto max-h-[min(42.5rem,calc(100dvh-2rem))] w-[min(34rem,calc(100vw-1.5rem))] overflow-hidden rounded-[1.125rem] border border-[var(--border)] bg-[var(--surface-solid)] p-0 text-[var(--page-text)] shadow-[var(--dialog-shadow)]"
  aria-labelledby="locale-dialog-heading"
  onclick={handleDialogClick}
>
  <div class="max-h-[min(42.5rem,calc(100dvh-2rem))] overflow-y-auto bg-[var(--surface-solid)]">
    <div class="sticky top-0 z-[2] bg-[var(--surface-solid)] px-6 pt-6 pb-3.5 shadow-[var(--sticky-shadow)]">
      <div class="mb-5 flex items-center justify-between gap-4">
        <h2 id="locale-dialog-heading" class="m-0 text-xl leading-6 text-[var(--page-text)]">
          {translate('nav.language', cur)}
        </h2>
        <button
          class="grid size-11 place-items-center rounded-[9px] border border-[var(--control-border)] bg-[var(--control-bg)] text-[1.45rem] text-[var(--page-text)]"
          type="button"
          aria-label={translate('nav.closeLanguage', cur)}
          onclick={closeDialog}
        >
          ×
        </button>
      </div>

      <label class="mb-[.4rem] block text-[.78rem] font-bold text-[var(--muted-text)]" for="locale-search">
        {translate('nav.searchLanguages', cur)}
      </label>
      <input
        class="min-h-11 w-full rounded-[.65rem] border border-[var(--field-border)] bg-[var(--field-bg)] px-[.8rem] py-[.7rem] text-[var(--page-text)] outline-none focus:border-[var(--accent-border)] focus:shadow-[0_0_0_3px_var(--focus-ring)]"
        id="locale-search"
        type="search"
        bind:value={search}
        autocomplete="off"
        spellcheck="false"
      />
    </div>

    <div class="px-6 pt-[.85rem] pb-6">
      <div class="grid grid-cols-2 gap-[.45rem] max-[520px]:grid-cols-1" translate="no">
        {#each filtered as code}
          <button
            type="button"
            class={[
              'flex min-h-12 items-center justify-between gap-[.7rem] rounded-[.65rem] border px-[.7rem] py-[.6rem] text-start text-[var(--page-text)] hover:border-[var(--accent-border)] hover:bg-[var(--hover-bg)]',
              code === cur
                ? 'border-[var(--accent-border)] bg-[var(--selected-bg)] shadow-[inset_0_0_0_1px_var(--accent-border)]'
                : 'border-[var(--control-border)] bg-[var(--control-bg)]'
            ]}
            aria-pressed={code === cur}
            onclick={() => choose(code)}
          >
            <span class="min-w-0 overflow-hidden text-ellipsis whitespace-nowrap" lang={code}>{displayNames[code]}</span>
            <small class="flex-none text-[.66rem] text-[var(--muted-text)] uppercase">{code}</small>
          </button>
        {/each}
      </div>

      {#if filtered.length === 0}
        <p class="text-center text-[var(--muted-text)]">{translate('nav.noLanguages', cur)}</p>
      {/if}
    </div>
  </div>
</dialog>

<style>
  .locale-dialog::backdrop {
    background: rgb(2 7 13 / .72);
    backdrop-filter: blur(2px);
  }
</style>
