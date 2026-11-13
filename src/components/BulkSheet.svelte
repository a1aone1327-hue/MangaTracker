<script lang="ts">
  import { errorText } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { LIBRARY_LABEL, librariesFor } from '../lib/format';
  import type { Library } from '../lib/types';
  import Sheet from './Sheet.svelte';

  let { kind, onclose }: { kind: 'move' | 'library' | 'delete'; onclose: () => void } = $props();

  let busy = $state(false);
  let error = $state('');

  const count = $derived(app.selected.size);
  const heading = $derived(
    kind === 'move' ? 'Перенести в группу' : kind === 'library' ? 'Сменить библиотеку' : 'Удалить выбранное?',
  );

  async function run(job: () => Promise<string>) {
    if (busy) return;
    busy = true;
    error = '';
    try {
      const message = await job();
      onclose();
      app.showToast(message);
    } catch (e) {
      error = errorText(e);
    } finally {
      busy = false;
    }
  }

  function moveTo(groupId: number | null, name: string) {
    void run(async () => {
      const moved = await app.moveSelected(groupId);
      return groupId === null ? `Без группы: ${moved}` : `Перенесено в «${name}»: ${moved}`;
    });
  }

  function changeLibrary(library: Library) {
    void run(async () => {
      const changed = await app.setLibrarySelected(library);
      return changed === 0
        ? `Уже на ${LIBRARY_LABEL[library]}`
        : `Теперь на ${LIBRARY_LABEL[library]}: ${changed}`;
    });
  }

  function confirmDelete() {
    onclose();
    app.deleteSelected();
  }
</script>

<Sheet title={heading} {onclose}>
  {#if kind === 'move'}
    <p class="lead">Выбрано тайтлов: {count}. Тапни группу, и они сразу переедут.</p>
    <div class="choices">
      {#each app.groups as g (g.id)}
        <button type="button" class="choice" style="--c: {g.color}" disabled={busy} onclick={() => moveTo(g.id, g.name)}>
          <span class="dot"></span>
          <span class="choice-name">{g.name}</span>
          <span class="choice-count">{app.counts.get(g.id) ?? 0}</span>
        </button>
      {/each}
      <button type="button" class="choice" style="--c: var(--muted)" disabled={busy} onclick={() => moveTo(null, '')}>
        <span class="dot"></span>
        <span class="choice-name">Без группы</span>
      </button>
    </div>
  {:else if kind === 'library'}
    <p class="lead">
      Выбрано тайтлов: {count}. Прогресс и группы сохранятся. Ссылки не меняются, новые можно
      вставить на экране тайтла.
    </p>
    <div class="choices">
      {#each librariesFor(app.medium) as lib (lib)}
        <button type="button" class="choice" disabled={busy} onclick={() => changeLibrary(lib)}>
          <span class="choice-name">{LIBRARY_LABEL[lib]}</span>
        </button>
      {/each}
    </div>
  {:else}
    <p class="lead">
      Будет удалено тайтлов: {count}. Сразу после удаления можно нажать «Отменить».
    </p>
    <div class="actions">
      <button type="button" class="btn danger solid" onclick={confirmDelete}>Удалить</button>
      <button type="button" class="btn" onclick={onclose}>Отмена</button>
    </div>
  {/if}

  {#if error}<p class="error" role="alert">{error}</p>{/if}
</Sheet>

<style>
  .lead {
    margin: 0 0 14px;
    color: var(--muted);
  }

  .choices {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .choice {
    display: flex;
    gap: 12px;
    align-items: center;
    min-height: 56px;
    padding: 0 16px;
    text-align: left;
    background: var(--surface-2);
    border: 1.5px solid var(--line);
    border-radius: 14px;
    transition:
      border-color 0.15s,
      transform 0.12s;
  }
  .choice:active {
    border-color: var(--accent);
    transform: scale(0.99);
  }
  .choice:disabled {
    opacity: 0.5;
  }

  .dot {
    flex: none;
    width: 12px;
    height: 12px;
    background: var(--c);
    border-radius: 50%;
  }

  .choice-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .choice-count {
    color: var(--muted);
    font-size: 0.85rem;
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .error {
    margin: 14px 0 0;
    color: var(--danger);
  }
</style>
