<script lang="ts">
  import { onMount } from 'svelte';
  import * as api from '../lib/api';
  import { app } from '../lib/app.svelte';
  import {
    KIND_LABEL,
    KIND_ORDER,
    LIBRARY_LABEL,
    counterLabel,
    librariesFor,
    parseCount,
  } from '../lib/format';
  import type { Counter, Library, MangaKind, TitleDraft } from '../lib/types';
  import { dataUrlToThumbnail } from '../lib/image';
  import CoverField from './CoverField.svelte';
  import Icon from './Icon.svelte';

  // Поля тайтла. В режиме правки каждое изменение сразу сохраняется через oncommit,
  // а кнопки +/− идут через onstep (мгновенно, прямо в базу). При создании ничего
  // не сохраняется, пока не нажата кнопка «Добавить».
  let {
    draft = $bindable(),
    oncommit,
    onstep,
  }: {
    draft: TitleDraft;
    oncommit?: () => void;
    onstep?: (counter: Counter, delta: -1 | 1) => void;
  } = $props();

  let nameInput: HTMLInputElement | undefined = $state();
  let detected = $state<Library | null>(null);
  let detectSeq = 0;
  let fetching = $state(false);

  export function focusName() {
    nameInput?.focus();
    nameInput?.scrollIntoView({ block: 'center' });
  }

  const libraries = $derived(
    librariesFor(draft.medium).includes(draft.library)
      ? librariesFor(draft.medium)
      : [...librariesFor(draft.medium), draft.library],
  );
  const counters = $derived<Counter[]>(draft.medium === 'anime' ? ['main'] : ['main', 'volumes']);
  const urlMismatch = $derived(detected !== null && detected !== draft.library);
  const blurred = $derived(app.blurAdult && draft.library === 'hentailib');

  // Ссылка определяет библиотеку. apply = false: только запомнить, не менять поля.
  async function detect(apply: boolean) {
    const seq = ++detectSeq;
    const lib = await api.detectLibrary(draft.url ?? '');
    if (seq !== detectSeq) return;
    detected = lib;
    if (apply && lib && lib !== draft.library) draft.library = lib;
  }

  onMount(() => {
    if (draft.url) void detect(false);
  });

  function commit() {
    oncommit?.();
  }

  function setLibrary(lib: Library) {
    if (draft.library === lib) return;
    draft.library = lib;
    commit();
  }

  function setKind(kind: MangaKind) {
    draft.kind = draft.kind === kind ? null : kind;
    commit();
  }

  function setGroup(id: number | null) {
    if (draft.group_id === id) return;
    draft.group_id = id;
    commit();
  }

  function doneOf(counter: Counter): number {
    return counter === 'main' ? draft.done_main : draft.done_volumes;
  }

  function totalOf(counter: Counter): number | null {
    return counter === 'main' ? draft.total_main : draft.total_volumes;
  }

  function setDone(counter: Counter, el: HTMLInputElement) {
    const n = parseCount(el.value) ?? 0;
    if (counter === 'main') draft.done_main = n;
    else draft.done_volumes = n;
    el.value = String(n);
    commit();
  }

  function setTotal(counter: Counter, el: HTMLInputElement) {
    const n = parseCount(el.value);
    if (counter === 'main') draft.total_main = n;
    else draft.total_volumes = n;
    el.value = n === null ? '' : String(n);
    commit();
  }

  function step(counter: Counter, delta: -1 | 1) {
    if (onstep) {
      onstep(counter, delta);
      return;
    }
    // при создании тайтла двигаем только черновик; границы проверит Rust при сохранении
    const next = Math.max(0, Math.min(doneOf(counter) + delta, totalOf(counter) ?? Infinity));
    if (counter === 'main') draft.done_main = next;
    else draft.done_volumes = next;
  }

  // «Подтянуть данные»: заполняем только то, что пусто, а количество глав обновляем
  // (оно меняется у онгоингов). Неудача не мешает: полей никто не трогает.
  async function fetchInfo() {
    const url = (draft.url ?? '').trim();
    if (!url || fetching) return;
    fetching = true;
    try {
      const info = await api.fetchTitleInfo(url);
      if (!info) {
        app.showToast('Не получилось получить данные, заполни вручную', 2400);
        return;
      }
      if (info.medium !== draft.medium) {
        app.showToast(info.medium === 'anime' ? 'Эта ссылка ведёт на аниме' : 'Эта ссылка ведёт на мангу');
        return;
      }
      if (!draft.name.trim()) draft.name = info.name;
      if (!draft.original_name?.trim()) draft.original_name = info.original_name;
      if (info.total_main !== null && info.total_main >= draft.done_main) {
        draft.total_main = info.total_main;
      }
      if (draft.kind === null && info.kind !== null) draft.kind = info.kind;
      draft.library = info.library;
      if (!draft.cover && info.cover_data) {
        try {
          draft.cover = await dataUrlToThumbnail(info.cover_data);
        } catch {
          // обложка не обязательна
        }
      }
      commit();
      app.showToast('Данные подтянуты', 1800);
    } catch {
      app.showToast('Не получилось получить данные, заполни вручную', 2400);
    } finally {
      fetching = false;
    }
  }

  async function openLink() {
    let url = (draft.url ?? '').trim();
    if (!url) return;
    if (!/^https?:\/\//i.test(url)) url = `https://${url}`;
    try {
      await api.openLink(url);
    } catch {
      app.showToast('Не удалось открыть ссылку');
    }
  }
</script>

<div class="fields">
  <label class="field">
    <span class="label">Название</span>
    <input
      bind:this={nameInput}
      class="input"
      value={draft.name}
      maxlength="200"
      autocomplete="off"
      placeholder="Как называется"
      oninput={(e) => (draft.name = e.currentTarget.value)}
      onchange={commit}
    />
  </label>

  <label class="field">
    <span class="label">Оригинальное название</span>
    <input
      class="input"
      value={draft.original_name ?? ''}
      autocomplete="off"
      placeholder="Необязательно"
      oninput={(e) => (draft.original_name = e.currentTarget.value)}
      onchange={commit}
    />
  </label>

  <div class="field">
    <span class="label">Группа</span>
    <div class="chips scroll" role="radiogroup" aria-label="Группа">
      <button
        type="button"
        class="chip"
        role="radio"
        aria-checked={draft.group_id === null}
        style="--c: var(--muted)"
        onclick={() => setGroup(null)}
      >
        <span class="dot"></span>Без группы
      </button>
      {#each app.groups as g (g.id)}
        <button
          type="button"
          class="chip"
          role="radio"
          aria-checked={draft.group_id === g.id}
          style="--c: {g.color}"
          onclick={() => setGroup(g.id)}
        >
          <span class="dot"></span>{g.name}
        </button>
      {/each}
    </div>
  </div>

  {#each counters as counter (counter)}
    <div class="field">
      <span class="label">{counterLabel(draft.medium, counter)}</span>
      <div class="counter">
        <button
          type="button"
          class="step"
          aria-label="Минус один: {counterLabel(draft.medium, counter)}"
          onclick={() => step(counter, -1)}
        >
          <Icon name="minus" />
        </button>
        <input
          class="num"
          inputmode="numeric"
          aria-label="Сколько пройдено: {counterLabel(draft.medium, counter)}"
          value={doneOf(counter)}
          onfocus={(e) => e.currentTarget.select()}
          onchange={(e) => setDone(counter, e.currentTarget)}
        />
        <span class="of">из</span>
        <input
          class="num total"
          inputmode="numeric"
          placeholder="?"
          aria-label="Сколько всего: {counterLabel(draft.medium, counter)}"
          value={totalOf(counter) ?? ''}
          onfocus={(e) => e.currentTarget.select()}
          onchange={(e) => setTotal(counter, e.currentTarget)}
        />
        <button
          type="button"
          class="step plus"
          aria-label="Плюс один: {counterLabel(draft.medium, counter)}"
          onclick={() => step(counter, 1)}
        >
          <Icon name="plus" />
        </button>
      </div>
    </div>
  {/each}

  <div class="field">
    <span class="label">Библиотека</span>
    <div class="chips" role="radiogroup" aria-label="Библиотека">
      {#each libraries as lib (lib)}
        <button
          type="button"
          class="chip"
          role="radio"
          aria-checked={draft.library === lib}
          onclick={() => setLibrary(lib)}
        >
          {LIBRARY_LABEL[lib]}
        </button>
      {/each}
    </div>
    {#if urlMismatch && detected}
      <p class="hint">
        Ссылка ведёт на {LIBRARY_LABEL[detected]}. Если тайтл переехал, вставь новую ссылку ниже.
      </p>
    {/if}
  </div>

  {#if draft.medium === 'manga'}
    <div class="field">
      <span class="label">Тип</span>
      <div class="chips" role="group" aria-label="Тип">
        {#each KIND_ORDER as kind (kind)}
          <button
            type="button"
            class="chip"
            aria-pressed={draft.kind === kind}
            onclick={() => setKind(kind)}
          >
            {KIND_LABEL[kind]}
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <div class="field">
    <span class="label">Ссылка на страницу</span>
    <div class="link-row">
      <input
        class="input"
        inputmode="url"
        autocomplete="off"
        placeholder="Вставь ссылку с сайта"
        value={draft.url ?? ''}
        oninput={(e) => {
          draft.url = e.currentTarget.value;
          void detect(true);
        }}
        onchange={commit}
      />
      {#if draft.url}
        <button type="button" class="icon-btn" aria-label="Открыть в браузере" onclick={openLink}>
          <Icon name="external" />
        </button>
      {/if}
    </div>
    {#if detected}
      <button type="button" class="btn fetch" disabled={fetching} onclick={fetchInfo}>
        {fetching ? 'Получаю данные…' : `Подтянуть данные с ${LIBRARY_LABEL[detected]}`}
      </button>
    {/if}
  </div>

  <div class="field">
    <span class="label">Обложка</span>
    <CoverField bind:cover={draft.cover} {blurred} oncommit={commit} />
  </div>

  <label class="field">
    <span class="label">Заметка</span>
    <textarea
      class="input note"
      rows="3"
      placeholder="Необязательно"
      value={draft.note}
      oninput={(e) => (draft.note = e.currentTarget.value)}
      onchange={commit}
    ></textarea>
  </label>
</div>

<style>
  .fields {
    display: block;
  }

  .counter {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .step {
    display: grid;
    flex: none;
    place-items: center;
    width: 52px;
    height: 52px;
    background: var(--surface-2);
    border: 1px solid var(--line);
    border-radius: 50%;
    transition: transform 0.12s;
  }
  .step:active {
    transform: scale(0.9);
  }
  .step.plus {
    color: #1a1230;
    background: var(--accent);
    border-color: transparent;
  }

  .num {
    flex: 1;
    min-width: 0;
    height: 52px;
    padding: 0 8px;
    color: var(--text);
    background: var(--bg);
    border: 1px solid var(--line);
    border-radius: 14px;
    font: inherit;
    font-size: 1.3rem;
    font-weight: 700;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }
  .num.total {
    font-weight: 600;
    color: var(--muted);
  }
  .num:focus {
    outline: 2px solid var(--accent);
    border-color: transparent;
  }

  .of {
    color: var(--muted);
  }

  .hint {
    margin: 10px 0 0;
    color: var(--accent);
    font-size: 0.9rem;
  }

  .fetch {
    width: 100%;
    margin-top: 10px;
  }

  .link-row {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .note {
    height: auto;
    padding: 12px 16px;
    line-height: 1.4;
    resize: vertical;
  }
</style>
