<script lang="ts">
  import { fly } from 'svelte/transition';
  import { app } from '../lib/app.svelte';
</script>

{#if app.toast}
  {@const t = app.toast}
  <div class="toast" role="status" transition:fly|global={{ y: 24, duration: 200 }}>
    <span class="text">{t.text}</span>
    {#if t.action}
      <button type="button" class="act" onclick={t.action.run}>{t.action.label}</button>
    {/if}
  </div>
{/if}

<style>
  .toast {
    position: fixed;
    left: 50%;
    bottom: calc(var(--safe-bottom) + 96px);
    z-index: 30;
    display: flex;
    gap: 14px;
    align-items: center;
    width: max-content;
    max-width: calc(100% - 40px);
    translate: -50% 0;
    padding: 12px 12px 12px 18px;
    background: var(--surface-2);
    border: 1px solid var(--line);
    border-radius: 14px;
    box-shadow: 0 8px 28px rgb(0 0 0 / 0.4);
  }

  .text {
    min-width: 0;
  }

  .act {
    flex: none;
    min-height: 40px;
    padding: 0 14px;
    color: var(--accent);
    border-radius: 10px;
    font-weight: 700;
  }
  .act:active {
    background: var(--surface);
  }
</style>
