<script lang="ts">
  import { app } from '../lib/app.svelte';

  const options = [
    { value: 'manga', label: 'Манга' },
    { value: 'anime', label: 'Аниме' },
  ] as const;

  const index = $derived(app.medium === 'manga' ? 0 : 1);
</script>

<div class="switch" role="tablist" aria-label="Раздел">
  <span class="pill" style="transform: translateX({index * 100}%)"></span>
  {#each options as o (o.value)}
    <button
      type="button"
      role="tab"
      aria-selected={app.medium === o.value}
      class:on={app.medium === o.value}
      onclick={() => app.setMedium(o.value)}
    >
      {o.label}
    </button>
  {/each}
</div>

<style>
  .switch {
    position: relative;
    display: grid;
    grid-template-columns: 1fr 1fr;
    padding: 4px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 999px;
  }

  .pill {
    position: absolute;
    top: 4px;
    bottom: 4px;
    left: 4px;
    width: calc(50% - 4px);
    background: var(--surface-2);
    border-radius: 999px;
    box-shadow: 0 0 0 1px var(--line);
    transition: transform 0.28s cubic-bezier(0.3, 0.8, 0.3, 1);
  }

  button {
    position: relative;
    z-index: 1;
    min-height: 44px;
    border-radius: 999px;
    color: var(--muted);
    font-weight: 650;
    transition: color 0.2s;
  }

  button.on {
    color: var(--text);
  }
</style>
