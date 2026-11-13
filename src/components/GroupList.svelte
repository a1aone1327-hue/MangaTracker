<script lang="ts">
  import { flip } from 'svelte/animate';
  import { slide } from 'svelte/transition';
  import { SvelteSet } from 'svelte/reactivity';
  import { app } from '../lib/app.svelte';
  import type { Group, Title } from '../lib/types';
  import Icon from './Icon.svelte';
  import TitleCard from './TitleCard.svelte';

  let {
    reorder,
    onedit,
    onadd,
    onopen,
  }: {
    reorder: boolean;
    onedit: (group: Group) => void;
    onadd: () => void;
    onopen: (title: Title) => void;
  } = $props();

  const UNGROUPED = -1;
  const collapsed = new SvelteSet<number>();

  function toggle(id: number) {
    if (collapsed.has(id)) collapsed.delete(id);
    else collapsed.add(id);
  }

  const titlesOf = (groupId: number | null) => app.visibleByGroup.get(groupId) ?? [];
  // при поиске раскрываем всё, чтобы найденное было видно, и прячем группы без совпадений
  const searching = $derived(app.tokens.length > 0);
  const shownGroups = $derived(
    reorder || !app.isFiltering ? app.groups : app.groups.filter((g) => titlesOf(g.id).length > 0),
  );
  const last = $derived(shownGroups.length - 1);
  const isOpen = (id: number) => searching || !collapsed.has(id);
  const nothingFound = $derived(
    app.isFiltering && app.visibleTitles.length === 0 && app.titles.length > 0,
  );
  const loose = $derived(titlesOf(null));

  function pickState(items: Title[]): 'none' | 'some' | 'all' {
    if (items.length === 0) return 'none';
    const n = items.filter((t) => app.selected.has(t.id)).length;
    return n === 0 ? 'none' : n === items.length ? 'all' : 'some';
  }

  function pickGroup(items: Title[]) {
    app.selectMany(
      items.map((t) => t.id),
      pickState(items) !== 'all',
    );
  }
</script>

{#snippet pickButton(items: Title[], name: string)}
  {@const state = pickState(items)}
  <button
    type="button"
    class="icon-btn"
    aria-label="Выбрать всё в группе «{name}»"
    disabled={items.length === 0}
    onclick={() => pickGroup(items)}
  >
    <span class="check" data-state={state}>
      {#if state === 'all'}<Icon name="check" size={18} />{:else if state === 'some'}<Icon name="minus" size={18} />{/if}
    </span>
  </button>
{/snippet}

{#snippet titleRows(list: Title[], color: string)}
  {#if list.length === 0}
    <p class="empty">Пока пусто</p>
  {:else}
    <ul class="titles">
      {#each list as t (t.id)}
        <li><TitleCard title={t} {color} {onopen} /></li>
      {/each}
    </ul>
  {/if}
{/snippet}

{#if searching && app.visibleTitles.length > 0}
  <p class="found">Найдено: {app.visibleTitles.length}</p>
{/if}

{#if nothingFound}
  <div class="none">
    <p>Ничего не найдено</p>
    <button type="button" class="btn" onclick={() => app.resetFilters()}>Сбросить поиск и фильтры</button>
  </div>
{/if}

{#if app.titles.length === 0}
  <section class="welcome">
    <strong>Здесь пока пусто</strong>
    <p>
      Нажми «+» внизу и добавь первый тайтл. Если вставить ссылку с MangaLib, HentaiLib или
      AnimeLib, библиотека определится сама, а название и обложку можно подтянуть кнопкой.
    </p>
    <p>
      Долгое нажатие на карточку включает режим выбора для массового переноса. Копию данных можно
      сделать в настройках.
    </p>
  </section>
{/if}

<ul class="list">
  {#each shownGroups as g, i (g.id)}
    {@const items = titlesOf(g.id)}
    <li class="group" style="--c: {g.color}" animate:flip={{ duration: 220 }}>
      <div class="head">
        <button
          type="button"
          class="toggle"
          aria-expanded={isOpen(g.id)}
          disabled={reorder || searching}
          onclick={() => toggle(g.id)}
        >
          <span class="stripe"></span>
          <span class="name">{g.name}</span>
          <span class="count">{items.length}</span>
          {#if !reorder}
            <span class="chev" class:open={isOpen(g.id)}><Icon name="chevron" size={20} /></span>
          {/if}
        </button>

        {#if reorder}
          <div class="arrows">
            <button
              type="button"
              class="icon-btn"
              aria-label="Поднять группу «{g.name}»"
              disabled={i === 0}
              onclick={() => app.moveGroup(g.id, -1)}
            >
              <Icon name="up" />
            </button>
            <button
              type="button"
              class="icon-btn"
              aria-label="Опустить группу «{g.name}»"
              disabled={i === last}
              onclick={() => app.moveGroup(g.id, 1)}
            >
              <Icon name="down" />
            </button>
          </div>
        {:else if app.selecting}
          {@render pickButton(items, g.name)}
        {:else}
          <button
            type="button"
            class="icon-btn"
            aria-label="Изменить группу «{g.name}»"
            onclick={() => onedit(g)}
          >
            <Icon name="edit" size={20} />
          </button>
        {/if}
      </div>

      {#if !reorder && isOpen(g.id)}
        <div transition:slide|global={{ duration: 200 }}>
          {@render titleRows(items, g.color)}
        </div>
      {/if}
    </li>
  {/each}

  {#if loose.length > 0}
    <li class="group" style="--c: var(--muted)">
      <div class="head">
        <button
          type="button"
          class="toggle"
          aria-expanded={isOpen(UNGROUPED)}
          disabled={reorder || searching}
          onclick={() => toggle(UNGROUPED)}
        >
          <span class="stripe"></span>
          <span class="name">Без группы</span>
          <span class="count">{loose.length}</span>
          {#if !reorder}
            <span class="chev" class:open={isOpen(UNGROUPED)}><Icon name="chevron" size={20} /></span>
          {/if}
        </button>
        {#if app.selecting}
          {@render pickButton(loose, 'Без группы')}
        {/if}
      </div>
      {#if !reorder && isOpen(UNGROUPED)}
        <div transition:slide|global={{ duration: 200 }}>
          {@render titleRows(loose, 'var(--muted)')}
        </div>
      {/if}
    </li>
  {/if}
</ul>

{#if !reorder && !app.isFiltering}
  <button type="button" class="add" onclick={onadd}>
    <Icon name="plus" size={20} />
    Новая группа
  </button>
{/if}

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .group {
    overflow: hidden;
    background: var(--surface);
    background: color-mix(in srgb, var(--c) 7%, var(--surface));
    border: 1px solid var(--line);
    border-radius: var(--radius);
  }

  .head {
    display: flex;
    align-items: center;
    padding-right: 4px;
  }

  .toggle {
    display: flex;
    flex: 1;
    align-items: center;
    gap: 12px;
    min-width: 0;
    min-height: 58px;
    text-align: left;
  }

  .stripe {
    align-self: stretch;
    width: 6px;
    margin-right: 4px;
    background: var(--c);
    border-radius: 0 6px 6px 0;
  }

  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    font-weight: 650;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .count {
    min-width: 28px;
    padding: 2px 9px;
    background: var(--surface-2);
    border-radius: 999px;
    color: var(--muted);
    font-size: 0.85rem;
    text-align: center;
  }

  .chev {
    display: grid;
    color: var(--muted);
    transition: transform 0.2s;
  }
  .chev.open {
    transform: rotate(90deg);
  }

  .arrows {
    display: flex;
  }

  .check {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    color: #1a1230;
    border: 2px solid var(--muted);
    border-radius: 50%;
    transition:
      background 0.15s,
      border-color 0.15s;
  }
  .check[data-state='all'] {
    background: var(--accent);
    border-color: var(--accent);
  }
  .check[data-state='some'] {
    color: var(--accent);
    border-color: var(--accent);
  }

  .icon-btn:disabled {
    opacity: 0.3;
  }

  .titles {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 0;
    padding: 0 10px 12px;
    list-style: none;
  }

  .empty {
    margin: 0;
    padding: 2px 22px 16px;
    color: var(--muted);
  }

  .found {
    margin: 0 0 10px;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .none {
    padding: 24px 0;
    text-align: center;
  }
  .none p {
    margin: 0 0 14px;
    color: var(--muted);
  }

  .welcome {
    padding: 16px 18px;
    margin-bottom: 14px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius);
  }
  .welcome p {
    margin: 8px 0 0;
    color: var(--muted);
    font-size: 0.95rem;
  }

  .add {
    display: flex;
    gap: 8px;
    align-items: center;
    justify-content: center;
    width: 100%;
    min-height: 56px;
    margin-top: 10px;
    color: var(--muted);
    border: 1.5px dashed var(--line);
    border-radius: var(--radius);
    transition:
      color 0.2s,
      border-color 0.2s,
      transform 0.12s;
  }
  .add:active {
    color: var(--text);
    border-color: var(--accent);
    transform: scale(0.99);
  }
</style>
