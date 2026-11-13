<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { SORT_HINT, SORT_LABEL, SORT_MODES, type SortMode } from '../lib/query';
  import Icon from './Icon.svelte';
  import Sheet from './Sheet.svelte';

  let { onclose }: { onclose: () => void } = $props();

  function pick(mode: SortMode) {
    app.setSort(mode);
    onclose();
  }
</script>

<Sheet title="Сортировка" {onclose}>
  <div class="choices" role="radiogroup" aria-label="Порядок тайтлов">
    {#each SORT_MODES as mode (mode)}
      <button
        type="button"
        class="choice"
        role="radio"
        aria-checked={app.sort === mode}
        onclick={() => pick(mode)}
      >
        <span class="text">
          <strong>{SORT_LABEL[mode]}</strong>
          <small>{SORT_HINT[mode]}</small>
        </span>
        {#if app.sort === mode}<Icon name="check" />{/if}
      </button>
    {/each}
  </div>
  <p class="note">
    Порядок обновляется при выборе сортировки и когда ты возвращаешься в приложение. Пока ты
    нажимаешь «+», карточки не прыгают из-под пальца. Тап по уже выбранному пункту пересортирует
    список сразу.
  </p>
</Sheet>

<style>
  .choices {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .choice {
    display: flex;
    gap: 12px;
    align-items: center;
    justify-content: space-between;
    min-height: 62px;
    padding: 8px 16px;
    text-align: left;
    background: var(--surface-2);
    border: 1.5px solid var(--line);
    border-radius: 14px;
    transition:
      border-color 0.15s,
      transform 0.12s;
  }
  .choice:active {
    transform: scale(0.99);
  }
  .choice[aria-checked='true'] {
    border-color: var(--accent);
  }

  .text strong,
  .text small {
    display: block;
  }
  .text small {
    margin-top: 2px;
    color: var(--muted);
  }

  .note {
    margin: 14px 0 0;
    color: var(--muted);
    font-size: 0.9rem;
  }
</style>
