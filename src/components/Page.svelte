<script lang="ts">
  import { onMount, type Snippet } from 'svelte';
  import { fly } from 'svelte/transition';
  import { pushBack } from '../lib/back';

  let { onclose, children }: { onclose: () => void; children: Snippet } = $props();

  // Системная кнопка «назад» закрывает страницу, а не приложение.
  onMount(() => pushBack(onclose));
</script>

<div class="page" transition:fly|global={{ x: 90, duration: 220 }}>
  <div class="inner">
    {@render children()}
  </div>
</div>

<style>
  .page {
    position: fixed;
    inset: 0;
    z-index: 10;
    overflow-y: auto;
    background: var(--bg);
  }

  .inner {
    max-width: 560px;
    margin: 0 auto;
    padding: calc(var(--safe-top) + 8px) 20px calc(var(--safe-bottom) + 40px);
  }
</style>
