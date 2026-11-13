<script lang="ts">
  import { errorText } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { LIBRARY_LABEL, formatDate, toDraft } from '../lib/format';
  import { closeTop } from '../lib/back';
  import type { Counter, Title, TitleDraft } from '../lib/types';
  import Icon from './Icon.svelte';
  import Page from './Page.svelte';
  import TitleFields from './TitleFields.svelte';

  let { title, onclose }: { title: Title; onclose: () => void } = $props();

  const id = title.id;
  let draft = $state<TitleDraft>(toDraft(title));
  const live = $derived(app.titles.find((t) => t.id === id) ?? title);

  // Каждое изменение поля сохраняется сразу. Ошибку (например, прочитано больше,
  // чем всего) показываем и возвращаем поля к последнему сохранённому состоянию.
  async function commit() {
    try {
      const saved = await app.updateTitle(id, $state.snapshot(draft));
      draft = toDraft(saved);
    } catch (e) {
      app.showToast(errorText(e));
      const current = app.titles.find((t) => t.id === id);
      if (current) draft = toDraft(current);
    }
  }

  async function step(counter: Counter, delta: -1 | 1) {
    try {
      const saved = await app.step(id, counter, delta);
      draft.done_main = saved.done_main;
      draft.done_volumes = saved.done_volumes;
    } catch (e) {
      app.showToast(errorText(e));
    }
  }

  function remove() {
    app.removeTitle(id);
    onclose();
  }
</script>

<Page {onclose}>
  <header class="bar">
    <button type="button" class="icon-btn" aria-label="Назад" onclick={closeTop}>
      <Icon name="back" />
    </button>
    <h2>{draft.name || 'Тайтл'}</h2>
    <button type="button" class="icon-btn danger" aria-label="Удалить тайтл" onclick={remove}>
      <Icon name="trash" />
    </button>
  </header>

  <TitleFields bind:draft oncommit={commit} onstep={step} />

  <footer class="meta">
    <p>Добавлен: {formatDate(live.created_at)}</p>
    {#if live.library_changed_at}
      <p>Перенесён в {LIBRARY_LABEL[live.library]}: {formatDate(live.library_changed_at)}</p>
    {/if}
  </footer>
</Page>

<style>
  .bar {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-bottom: 18px;
  }

  h2 {
    flex: 1;
    min-width: 0;
    margin: 0;
    overflow: hidden;
    font-size: 1.15rem;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .danger {
    color: var(--danger);
  }

  .meta {
    margin-top: 8px;
    color: var(--muted);
    font-size: 0.85rem;
  }
  .meta p {
    margin: 0 0 4px;
  }
</style>
