<script lang="ts">
  import { app } from '../lib/app.svelte';
  import Icon from './Icon.svelte';

  let { onsort }: { onsort: () => void } = $props();
</script>

<div class="row">
  <div class="search">
    <Icon name="search" size={20} />
    <input
      value={app.query}
      placeholder="Поиск по названию"
      aria-label="Поиск по названию"
      enterkeyhint="search"
      autocomplete="off"
      autocapitalize="off"
      spellcheck="false"
      oninput={(e) => (app.query = e.currentTarget.value)}
      onkeydown={(e) => {
        if (e.key === 'Enter') e.currentTarget.blur();
      }}
    />
    {#if app.query}
      <button type="button" class="clear" aria-label="Очистить поиск" onclick={() => (app.query = '')}>
        <Icon name="close" size={18} />
      </button>
    {/if}
  </div>

  <button type="button" class="icon-btn sort" aria-label="Сортировка" onclick={onsort}>
    <Icon name="sortlines" />
    {#if app.sort !== 'name'}<span class="badge"></span>{/if}
  </button>
</div>

<style>
  .row {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-top: 14px;
  }

  .search {
    display: flex;
    flex: 1;
    gap: 10px;
    align-items: center;
    min-width: 0;
    min-height: var(--tap);
    padding: 0 6px 0 16px;
    color: var(--muted);
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 999px;
    transition: border-color 0.15s;
  }
  .search:focus-within {
    border-color: var(--accent);
  }

  input {
    flex: 1;
    min-width: 0;
    height: var(--tap);
    color: var(--text);
    background: none;
    border: 0;
    font: inherit;
  }
  input:focus {
    outline: none;
  }

  .clear {
    display: grid;
    flex: none;
    place-items: center;
    width: 40px;
    height: 40px;
    color: var(--muted);
    border-radius: 50%;
  }
  .clear:active {
    background: var(--surface-2);
  }

  .sort {
    position: relative;
  }

  .badge {
    position: absolute;
    top: 10px;
    right: 10px;
    width: 8px;
    height: 8px;
    background: var(--accent);
    border-radius: 50%;
  }
</style>
