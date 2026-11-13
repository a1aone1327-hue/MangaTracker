<script lang="ts">
  import { app } from '../lib/app.svelte';
  import type { BulkAction } from '../lib/types';
  import Icon from './Icon.svelte';

  let { onaction }: { onaction: (action: BulkAction) => void } = $props();

  const none = $derived(app.selected.size === 0);
</script>

<nav class="bar" aria-label="Действия с выбранным">
  <button type="button" class="act" disabled={none} onclick={() => onaction('move')}>
    <Icon name="folder" />
    <span>В группу</span>
  </button>
  <button type="button" class="act" disabled={none} onclick={() => onaction('library')}>
    <Icon name="swap" />
    <span>Библиотека</span>
  </button>
  <button type="button" class="act danger" disabled={none} onclick={() => onaction('delete')}>
    <Icon name="trash" />
    <span>Удалить</span>
  </button>
</nav>

<style>
  .bar {
    position: fixed;
    right: 0;
    bottom: 0;
    left: 0;
    z-index: 9;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    max-width: 560px;
    margin: 0 auto;
    padding: 6px 8px calc(var(--safe-bottom) + 6px);
    background: var(--surface);
    border: 1px solid var(--line);
    border-bottom: 0;
    border-radius: 20px 20px 0 0;
    box-shadow: 0 -8px 24px rgb(0 0 0 / 0.35);
  }

  .act {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: center;
    justify-content: center;
    min-height: 62px;
    border-radius: 14px;
    font-size: 0.8rem;
    font-weight: 600;
    transition:
      background 0.15s,
      transform 0.12s;
  }
  .act:active {
    background: var(--surface-2);
    transform: scale(0.96);
  }
  .act:disabled {
    opacity: 0.35;
    pointer-events: none;
  }
  .act.danger {
    color: var(--danger);
  }
</style>
