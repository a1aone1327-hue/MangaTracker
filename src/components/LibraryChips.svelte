<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { LIBRARY_LABEL } from '../lib/format';
</script>

{#if app.libraryChips.length > 1}
  <div class="chips scroll libs" role="group" aria-label="Библиотека">
    <button
      type="button"
      class="chip"
      aria-pressed={app.activeFilter === 'all'}
      onclick={() => (app.libraryFilter = 'all')}
    >
      Все <small>{app.titles.length}</small>
    </button>
    {#each app.libraryChips as c (c.library)}
      <button
        type="button"
        class="chip"
        aria-pressed={app.activeFilter === c.library}
        onclick={() => (app.libraryFilter = c.library)}
      >
        {LIBRARY_LABEL[c.library]} <small>{c.count}</small>
      </button>
    {/each}
  </div>
{/if}

<style>
  .libs {
    margin-top: 12px;
  }
</style>
