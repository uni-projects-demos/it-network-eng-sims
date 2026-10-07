<script lang="ts">
import ButtonGH from "#lib/components/ButtonGH.svelte";
import ButtonLocale from "#lib/components/ButtonLocale.svelte";
import ButtonTheme from "#lib/components/ButtonTheme.svelte";
import SimTCP from "#lib/components/transport/SimTCP.svelte";
import SimUDP from "#lib/components/transport/SimUDP.svelte";
import { locale, translate } from "#lib/i18n/index.ts";
import { sims } from "#lib/sims/registry.ts";

type Sim = (typeof sims)[number];
type SimId = Sim["id"];
type TranslateFn = (key: string) => string;

let tooltipSim: Sim | null = null;
let tooltipX: number = 0;
let tooltipY: number = 0;
let selectedSim: SimId = "tcp";

let t: TranslateFn;
$: t = (key: string): string => translate(`nav.${key}`, $locale);

function showTooltip(sim: Sim, event: MouseEvent | FocusEvent): void {
  const element: HTMLElement = event.currentTarget as HTMLElement;
  const rect: DOMRect = element.getBoundingClientRect();

  tooltipSim = sim;
  tooltipX = rect.left + rect.width / 2;
  tooltipY = rect.bottom + 8;
}

function hideTooltip(): void {
  tooltipSim = null;
}
</script>

<svelte:head>
  <title>{t('siteTitle')}</title>
</svelte:head>

<div class="h-dvh overflow-hidden max-[1180px]:h-auto max-[1180px]:min-h-dvh max-[1180px]:overflow-visible">
  <header
    class="relative z-20 flex h-[78px] items-center gap-3.5 border-b border-[var(--divider)] bg-[var(--header-bg)] px-5 py-3 text-[var(--header-text)] backdrop-blur-[14px] max-[1180px]:h-auto max-[1180px]:min-h-[78px] max-[1180px]:px-3.5 max-[820px]:flex-wrap max-[820px]:gap-2 max-[480px]:p-2"
  >
    <div class="flex min-w-0 flex-none items-center gap-2.5 max-[820px]:basis-full max-[820px]:grow">
      <div
        class="grid size-11 flex-none place-items-center rounded-[9px] border border-[var(--header-control-border)] bg-[var(--header-control-bg)] text-[var(--header-control-text)]"
        aria-hidden="true"
      >
        <svg class="size-[22px] overflow-visible" viewBox="0 0 24 24">
          <path
            class="fill-none stroke-current stroke-[1.8] [stroke-linecap:round] [stroke-linejoin:round]"
            d="M12 5.2 5.7 17.4M12 5.2l6.3 12.2M5.7 17.4h12.6"
          />
          <circle class="fill-[var(--header-control-bg)] stroke-current stroke-[1.8]" cx="12" cy="5.2" r="2.15" />
          <circle class="fill-[var(--header-control-bg)] stroke-current stroke-[1.8]" cx="5.7" cy="17.4" r="2.15" />
          <circle class="fill-[var(--header-control-bg)] stroke-current stroke-[1.8]" cx="18.3" cy="17.4" r="2.15" />
        </svg>
      </div>

      <h1
        class="m-0 whitespace-nowrap text-lg tracking-[-.02em] text-[var(--header-text)] max-[1180px]:text-base max-[700px]:whitespace-normal max-[700px]:text-[15px] max-[700px]:leading-[1.15]"
      >
        {t('siteTitle')}
      </h1>
    </div>

    <div class="flex min-w-0 flex-1 items-center gap-2 max-[820px]:w-full max-[820px]:basis-full">
      <nav
        class="
          flex min-w-0 flex-nowrap items-center gap-1.5
          overflow-x-auto overflow-y-hidden
          max-[820px]:flex-1
          max-[480px]:gap-1
        "
        aria-label={t('simulationSelector')}
      >
      {#each sims as sim}
        <button
          class={[
            'grid h-11 flex-none content-center gap-px rounded-[10px] border px-2.5 py-[5px] text-start',
            'text-(--header-text) transition-colors',
            sim.status === 'available'
              ? 'cursor-pointer hover:border-(--header-control-border) hover:bg-(--header-control-hover)'
              : 'cursor-not-allowed opacity-60',
            selectedSim === sim.id
              ? 'border-(--header-control-border) bg-(--header-control-hover)'
              : 'border-transparent bg-transparent'
          ]}
          aria-disabled={sim.status !== 'available'}
          aria-describedby={`tooltip-${sim.id}`}
          onmouseenter={(event: MouseEvent): void => showTooltip(sim, event)}
          onmouseleave={(): void => hideTooltip()}
          onfocus={(event: FocusEvent): void => showTooltip(sim, event)}
          onblur={(): void => hideTooltip()}
          onclick={(): void => {
            if (sim.status === 'available') {
              selectedSim = sim.id;
            }
          }}
        >
          <small
            class="text-[8px] leading-[1.1] tracking-[.08em] text-(--header-muted) uppercase"
          >
            {t(sim.groupKey)}
          </small>

          <strong class="whitespace-nowrap text-[11px] leading-[1.15] font-bold">
            {t(sim.nameKey)}
          </strong>
        </button>
      {/each}
      </nav>

      <div class="ms-auto flex h-11 flex-none items-center gap-2" aria-label={t('sitePreferences')}>
        <ButtonTheme />
        <ButtonLocale />
        <ButtonGH />
      </div>
    </div>
  </header>

  {#if tooltipSim}
    <div
      id={`tooltip-${tooltipSim.id}`}
      role="tooltip"
      class="
        pointer-events-none fixed z-50
        w-max max-w-64
        -translate-x-1/2
        rounded-lg
        border border-(--header-control-border)
        bg-(--surface-solid)
        px-3 py-2
        text-[11px] leading-4 text-(--page-text)
        shadow-lg
      "
      style={`left: clamp(132px, ${tooltipX}px, calc(100vw - 132px)); top: ${tooltipY}px;`}
    >
      <div class="font-semibold">
        {t(tooltipSim.nameKey)}
      </div>

      <div class="mt-0.5 text-(--muted-text)">
        {t(tooltipSim.descriptionKey)}
      </div>

      {#if tooltipSim.status !== 'available'}
        <div class="mt-1 text-[10px] font-semibold text-(--danger)">
          {t('planned')}
        </div>
      {/if}

      <div
        class="
          absolute bottom-full left-1/2
          size-2
          -translate-x-1/2 translate-y-1 rotate-45
          border-t border-l border-(--header-control-border)
          bg-(--surface-solid)
        "
        aria-hidden="true"
      ></div>
    </div>
  {/if}

  <div class="h-[calc(100dvh-78px)] min-h-0 max-[1180px]:h-auto">
    <main
      class="h-full min-w-0 overflow-hidden p-3 max-[1180px]:h-auto max-[1180px]:min-h-[calc(100dvh-78px)] max-[1180px]:overflow-visible max-[700px]:p-2"
    >
      {#if selectedSim === 'tcp'}
        <SimTCP />
      {:else if selectedSim === 'udp'}
        <SimUDP />
      {/if}
    </main>
  </div>
</div>
