<script lang="ts">
  import { onMount } from 'svelte';
  import { app } from './lib/app.svelte';
  import type { BulkAction, Group, Title } from './lib/types';
  import BulkSheet from './components/BulkSheet.svelte';
  import GroupList from './components/GroupList.svelte';
  import GroupSheet from './components/GroupSheet.svelte';
  import ImportSheet from './components/ImportSheet.svelte';
  import Icon from './components/Icon.svelte';
  import LibraryChips from './components/LibraryChips.svelte';
  import SearchBar from './components/SearchBar.svelte';
  import SectionSwitch from './components/SectionSwitch.svelte';
  import SelectionBar from './components/SelectionBar.svelte';
  import SettingsSheet from './components/SettingsSheet.svelte';
  import SortSheet from './components/SortSheet.svelte';
  import TitleScreen from './components/TitleScreen.svelte';
  import TitleSheet from './components/TitleSheet.svelte';
  import Toast from './components/Toast.svelte';

  let groupSheet = $state<{ group: Group | null } | null>(null);
  let titleSheet = $state(false);
  let settings = $state(false);
  let importing = $state<{ fileName: string; json: string } | null>(null);
  let sortSheet = $state(false);
  let openTitle = $state<Title | null>(null);
  let bulk = $state<BulkAction | null>(null);
  let reorder = $state(false);

  const showFab = $derived(app.ready && !app.failure && !reorder && !app.selecting);
  const allPicked = $derived(
    app.visibleTitles.length > 0 && app.visibleTitles.every((t) => app.selected.has(t.id)),
  );

  function toggleAll() {
    if (allPicked) app.selected.clear();
    else app.selectAllVisible();
  }

  onMount(() => {
    app.init();
    void app.load();
  });
</script>

<main>
  {#if app.selecting}
    <header>
      <button type="button" class="icon-btn" aria-label="Выйти из режима выбора" onclick={() => app.endSelect()}>
        <Icon name="close" />
      </button>
      <h1 class="picked">Выбрано: {app.selected.size}</h1>
      <button type="button" class="btn all" onclick={toggleAll}>{allPicked ? 'Снять' : 'Все'}</button>
    </header>
  {:else}
    <header>
      <h1>Библиотека</h1>
      <div class="tools">
        {#if reorder}
          <button type="button" class="btn primary all" onclick={() => (reorder = false)}>Готово</button>
        {:else}
          {#if app.groups.length > 1}
            <button
              type="button"
              class="icon-btn"
              aria-label="Изменить порядок групп"
              onclick={() => (reorder = true)}
            >
              <Icon name="sort" />
            </button>
          {/if}
          <button type="button" class="icon-btn" aria-label="Настройки" onclick={() => (settings = true)}>
            <Icon name="settings" />
          </button>
        {/if}
      </div>
    </header>
  {/if}

  <SectionSwitch />
  <SearchBar onsort={() => (sortSheet = true)} />
  <LibraryChips />

  <div class="content">
    {#if app.failure}
      <section class="problem" role="alert">
        <strong>Не удалось открыть данные</strong>
        <p>{app.failure}</p>
        <p class="note">
          Данные на телефоне не тронуты. Перезапусти приложение; если не помогло, установи свежую
          версию APK.
        </p>
        <button type="button" class="btn" onclick={() => app.load()}>Повторить</button>
      </section>
    {:else if app.ready}
      <GroupList
        {reorder}
        onedit={(group) => (groupSheet = { group })}
        onadd={() => (groupSheet = { group: null })}
        onopen={(title) => (openTitle = title)}
      />
    {/if}
  </div>
</main>

{#if showFab}
  <button type="button" class="fab" aria-label="Добавить тайтл" onclick={() => (titleSheet = true)}>
    <Icon name="plus" size={28} />
  </button>
{/if}

{#if app.selecting}
  <SelectionBar onaction={(action) => (bulk = action)} />
{/if}

{#if openTitle}
  {#key openTitle.id}
    <TitleScreen title={openTitle} onclose={() => (openTitle = null)} />
  {/key}
{/if}

{#if groupSheet}
  {#key groupSheet.group?.id ?? 'new'}
    <GroupSheet group={groupSheet.group} onclose={() => (groupSheet = null)} />
  {/key}
{/if}

{#if titleSheet}
  <TitleSheet onclose={() => (titleSheet = false)} />
{/if}

{#if settings}
  <SettingsSheet
    onclose={() => (settings = false)}
    onimport={(fileName, json) => {
      settings = false;
      importing = { fileName, json };
    }}
  />
{/if}

{#if importing}
  <ImportSheet fileName={importing.fileName} json={importing.json} onclose={() => (importing = null)} />
{/if}

{#if sortSheet}
  <SortSheet onclose={() => (sortSheet = false)} />
{/if}

{#if bulk}
  <BulkSheet kind={bulk} onclose={() => (bulk = null)} />
{/if}

<Toast />

<style>
  main {
    max-width: 560px;
    margin: 0 auto;
    padding: calc(var(--safe-top) + 20px) 20px calc(var(--safe-bottom) + 120px);
  }

  header {
    display: flex;
    gap: 12px;
    align-items: center;
    justify-content: space-between;
    min-height: var(--tap);
    margin-bottom: 14px;
  }

  h1 {
    margin: 0;
    font-size: 2rem;
    font-weight: 750;
    letter-spacing: -0.02em;
  }
  h1.picked {
    flex: 1;
    font-size: 1.4rem;
  }

  .tools {
    display: flex;
    gap: 2px;
    align-items: center;
  }

  .all {
    min-height: 40px;
    padding: 0 18px;
  }

  .content {
    margin-top: 16px;
  }

  .problem {
    padding: 18px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius);
  }

  .problem p {
    margin: 6px 0 10px;
    color: var(--muted);
  }
  .problem .note {
    margin-bottom: 14px;
    font-size: 0.9rem;
  }

  .fab {
    position: fixed;
    right: 20px;
    bottom: calc(var(--safe-bottom) + 24px);
    z-index: 8;
    display: grid;
    place-items: center;
    width: 60px;
    height: 60px;
    color: #1a1230;
    background: var(--accent);
    border-radius: 50%;
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.45);
    transition: transform 0.12s;
  }
  .fab:active {
    transform: scale(0.92);
  }
</style>
