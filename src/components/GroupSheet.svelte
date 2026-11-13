<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { errorText } from '../lib/api';
  import { PALETTE } from '../lib/palette';
  import type { Group, OnDelete } from '../lib/types';
  import Icon from './Icon.svelte';
  import Sheet from './Sheet.svelte';

  // group = null значит «создать новую»
  let { group = null, onclose }: { group?: Group | null; onclose: () => void } = $props();

  let name = $state(group?.name ?? '');
  let color = $state(
    group?.color ?? PALETTE.find((c) => !app.groups.some((g) => g.color === c)) ?? PALETTE[0],
  );
  let error = $state('');
  let busy = $state(false);
  let view = $state<'form' | 'delete'>('form');
  let target = $state<number | 'none' | null>(null);

  const others = $derived(app.groups.filter((g) => g.id !== group?.id));
  const titleCount = $derived(group ? (app.counts.get(group.id) ?? 0) : 0);

  function focusOnMount(node: HTMLInputElement, enabled: boolean) {
    if (enabled) node.focus();
  }

  async function run(action: () => Promise<void>) {
    if (busy) return;
    busy = true;
    error = '';
    try {
      await action();
    } catch (e) {
      error = errorText(e);
    } finally {
      busy = false;
    }
  }

  function save() {
    void run(async () => {
      if (group) await app.updateGroup(group.id, name, color);
      else await app.createGroup(name, color);
      onclose();
    });
  }

  function startDelete() {
    const g = group;
    if (!g) return;
    if (titleCount === 0) {
      // переносить нечего: удаляем сразу, без лишних вопросов
      void run(async () => {
        await app.deleteGroup(g.id, { action: 'ungroup' });
        app.showToast(`Группа «${g.name}» удалена`);
        onclose();
      });
      return;
    }
    target = others.length === 0 ? 'none' : null;
    view = 'delete';
  }

  function confirmDelete() {
    const g = group;
    const chosen = target;
    if (!g || chosen === null) return;
    const onDelete: OnDelete =
      chosen === 'none' ? { action: 'ungroup' } : { action: 'move_to', group_id: chosen };
    const destination = chosen === 'none' ? null : others.find((o) => o.id === chosen);
    void run(async () => {
      const moved = await app.deleteGroup(g.id, onDelete);
      app.showToast(
        destination
          ? `Группа удалена. Перенесено в «${destination.name}»: ${moved}`
          : `Группа удалена. Осталось без группы: ${moved}`,
      );
      onclose();
    });
  }
</script>

<Sheet title={view === 'delete' ? 'Удаление группы' : group ? 'Группа' : 'Новая группа'} {onclose}>
  {#if view === 'form'}
    <form
      onsubmit={(e) => {
        e.preventDefault();
        save();
      }}
    >
      <label class="field">
        <span class="label">Название</span>
        <input
          class="input"
          bind:value={name}
          maxlength="40"
          autocomplete="off"
          placeholder="Например, Перечитываю"
          use:focusOnMount={!group}
        />
      </label>

      <div class="field">
        <span class="label">Цвет</span>
        <div class="swatches" role="radiogroup" aria-label="Цвет группы">
          {#each PALETTE as c (c)}
            <button
              type="button"
              class="swatch"
              role="radio"
              aria-checked={color === c}
              aria-label="Цвет {c}"
              style="--c: {c}"
              onclick={() => (color = c)}
            >
              {#if color === c}<Icon name="check" size={20} />{/if}
            </button>
          {/each}
        </div>
      </div>

      {#if error}<p class="error" role="alert">{error}</p>{/if}

      <div class="actions">
        <button class="btn primary" type="submit" disabled={busy || !name.trim()}>
          {group ? 'Сохранить' : 'Создать'}
        </button>
        {#if group}
          <button class="btn danger" type="button" disabled={busy} onclick={startDelete}>
            Удалить группу
          </button>
        {/if}
      </div>
    </form>
  {:else if group}
    <p class="lead">В группе «{group.name}» тайтлов: {titleCount}. Они не пропадут. Куда их перенести?</p>

    <div class="choices" role="radiogroup" aria-label="Куда перенести тайтлы">
      {#each others as o (o.id)}
        <button
          type="button"
          class="choice"
          role="radio"
          aria-checked={target === o.id}
          style="--c: {o.color}"
          onclick={() => (target = o.id)}
        >
          <span class="dot"></span>
          <span class="choice-name">{o.name}</span>
          <span class="choice-count">{app.counts.get(o.id) ?? 0}</span>
        </button>
      {/each}
      <button
        type="button"
        class="choice"
        role="radio"
        aria-checked={target === 'none'}
        style="--c: var(--muted)"
        onclick={() => (target = 'none')}
      >
        <span class="dot"></span>
        <span class="choice-name">Оставить без группы</span>
      </button>
    </div>

    {#if error}<p class="error" role="alert">{error}</p>{/if}

    <div class="actions">
      <button
        class="btn danger solid"
        type="button"
        disabled={busy || target === null}
        onclick={confirmDelete}
      >
        Удалить «{group.name}»
      </button>
    </div>
  {/if}
</Sheet>

<style>
  .field {
    display: block;
    margin-bottom: 18px;
  }

  .label {
    display: block;
    margin-bottom: 8px;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .swatches {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 10px;
  }

  .swatch {
    display: grid;
    place-items: center;
    aspect-ratio: 1;
    color: #151826;
    background: var(--c);
    border-radius: 50%;
    transition: transform 0.12s;
  }
  .swatch:active {
    transform: scale(0.9);
  }
  .swatch[aria-checked='true'] {
    box-shadow:
      0 0 0 3px var(--surface),
      0 0 0 5px var(--c);
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 6px;
  }

  .error {
    margin: 0 0 14px;
    color: var(--danger);
  }

  .lead {
    margin: 0 0 14px;
    color: var(--muted);
  }

  .choices {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 18px;
  }

  .choice {
    display: flex;
    gap: 12px;
    align-items: center;
    min-height: 54px;
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
    transform: scale(0.99);
  }
  .choice[aria-checked='true'] {
    border-color: var(--accent);
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
</style>
