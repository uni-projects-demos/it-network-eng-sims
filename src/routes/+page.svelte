<script lang="ts">
import ButtonGH from "#lib/components/ButtonGH.svelte";
import ButtonLocale from "#lib/components/ButtonLocale.svelte";
import ButtonTheme from "#lib/components/ButtonTheme.svelte";
import SimTCP from "#lib/components/transport/SimTCP.svelte";
import SimUDP from "#lib/components/transport/SimUDP.svelte";
import SimRouting from "#lib/components/network/SimRouting.svelte";
import SimFlow from "#lib/components/network/SimFlow.svelte";
import SimWSN from "#lib/components/network/SimWSN.svelte";
import { onMount } from "svelte";
import { locale, translate } from "#lib/i18n/index.ts";
import { sims } from "#lib/sims/registry.ts";

type Sim = (typeof sims)[number];
type SimId = Sim["id"];
type TranslateFn = (key: string) => string;

let tooltipSim: Sim | null = null;
let tooltipX: number = 0;
let tooltipY: number = 0;
let selectedSim: SimId = "wsn";

onMount(() => {
  const selectHash = (): void => {
    const id = window.location.hash.slice(1);
    const sim = sims.find((candidate) => candidate.id === id && candidate.status === "available");
    if (sim) selectedSim = sim.id;
  };
  selectHash();
  window.addEventListener("hashchange", selectHash);
  return () => window.removeEventListener("hashchange", selectHash);
});

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

<div class={[
  "h-dvh overflow-hidden max-[1180px]:h-auto max-[1180px]:min-h-dvh max-[1180px]:overflow-visible",
  (selectedSim === "rip" || selectedSim === "ospf" || selectedSim === "flow" || selectedSim === "wsn") && "max-[1240px]:h-auto max-[1240px]:min-h-dvh max-[1240px]:overflow-visible",
]}>
  <header class="site-header">
    <div class="site-header__brand">
      <div class="site-header__mark" aria-hidden="true">
        <svg viewBox="0 0 24 24">
          <path d="M12 5.2 5.7 17.4M12 5.2l6.3 12.2M5.7 17.4h12.6"/>
          <circle cx="12" cy="5.2" r="2.15"/>
          <circle cx="5.7" cy="17.4" r="2.15"/>
          <circle cx="18.3" cy="17.4" r="2.15"/>
        </svg>
      </div>
      <h1 class="site-header__title">{t('siteTitle')}</h1>
    </div>
    <nav class="site-header__nav" aria-label={t('simulationSelector')}>
      {#each sims as sim (sim.id)}
        <button type="button" class="site-header__module"
          aria-disabled={sim.status !== 'available'} aria-pressed={selectedSim === sim.id}
          aria-describedby={`tooltip-${sim.id}`}
          onmouseenter={(event: MouseEvent): void => showTooltip(sim, event)}
          onmouseleave={hideTooltip}
          onfocus={(event: FocusEvent): void => showTooltip(sim, event)}
          onblur={hideTooltip}
          onclick={(): void => {
            if (sim.status === 'available') {
              selectedSim = sim.id;
              window.location.hash = sim.id;
            }
          }}>
          <small>{t(sim.groupKey)}</small>
          <strong>{t(sim.nameKey)}</strong>
        </button>
      {/each}
    </nav>
    <div class="site-header__prefs" aria-label={t('sitePreferences')}>
      <ButtonTheme/>
      <ButtonLocale/>
      <ButtonGH/>
    </div>
  </header>

  {#if tooltipSim}
    <div id={`tooltip-${tooltipSim.id}`} role="tooltip" class="site-header__tooltip"
      style={`left: clamp(132px, ${tooltipX}px, calc(100vw - 132px)); top: ${tooltipY}px;`}>
      <div class="site-header__tooltip-title">{t(tooltipSim.nameKey)}</div>
      <div class="site-header__tooltip-description">{t(tooltipSim.descriptionKey)}</div>
      {#if tooltipSim.status !== 'available'}
        <div class="site-header__tooltip-planned">{t('planned')}</div>
      {/if}
      <div class="site-header__tooltip-arrow" aria-hidden="true"></div>
    </div>
  {/if}

  <div class={[
    "h-[calc(100dvh-78px)] min-h-0 max-[1180px]:h-auto",
    (selectedSim === "rip" || selectedSim === "ospf" || selectedSim === "flow" || selectedSim === "wsn") && "max-[1240px]:h-auto",
  ]}>
    <main
      class={[
        "h-full min-w-0 overflow-hidden p-3 max-[1180px]:h-auto max-[1180px]:min-h-[calc(100dvh-78px)] max-[1180px]:overflow-visible max-[700px]:p-2",
        (selectedSim === "rip" || selectedSim === "ospf" || selectedSim === "flow" || selectedSim === "wsn") && "max-[1240px]:h-auto max-[1240px]:min-h-[calc(100dvh-78px)] max-[1240px]:overflow-visible",
      ]}
    >
      {#if selectedSim === 'tcp'}
        <SimTCP />
      {:else if selectedSim === 'udp'}
        <SimUDP />
      {:else if selectedSim === 'rip'}
        <SimRouting protocol="rip" />
      {:else if selectedSim === 'ospf'}
        <SimRouting protocol="ospf" />
      {:else if selectedSim === 'wsn'}
        <SimWSN />
      {:else if selectedSim === 'flow'}
        <SimFlow />
      {/if}
    </main>
  </div>
</div>
