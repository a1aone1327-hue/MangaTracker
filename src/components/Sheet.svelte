<script lang="ts">
  import { onMount, type Snippet } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { closeTop, pushBack } from '../lib/back';

  let { title, onclose, children }: { title: string; onclose: () => void; children: Snippet } =
    $props();

  // Кнопка «назад» и тап по затемнению закрывают шторку одним и тем же путём.
  onMount(() => pushBack(onclose));
</script>

<button
  type="button"
  class="backdrop"
  aria-label="Закрыть"
  onclick={closeTop}
  transition:fade|global={{ duration: 160 }}
></button>

<section
  class="sheet"
  role="dialog"
  aria-modal="true"
  aria-label={title}
  transition:fly|global={{ y: 360, duration: 240 }}
>
  <div class="grab"></div>
  <h2>{title}</h2>
  {@render children()}
</section>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 20;
    background: rgb(6 7 12 / 0.62);
  }

  .sheet {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 21;
    max-width: 560px;
    max-height: 88dvh;
    margin: 0 auto;
    overflow-y: auto;
    padding: 10px 20px calc(var(--safe-bottom) + 20px);
    background: var(--surface);
    border: 1px solid var(--line);
    border-bottom: 0;
    border-radius: 24px 24px 0 0;
  }

  .grab {
    width: 40px;
    height: 4px;
    margin: 0 auto 14px;
    border-radius: 2px;
    background: var(--line);
  }

  h2 {
    margin: 0 0 16px;
    font-size: 1.2rem;
    letter-spacing: -0.01em;
  }
</style>
