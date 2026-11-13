<script lang="ts">
  import { fly } from 'svelte/transition';
  import { errorText } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { LIBRARY_LABEL, isComplete, percentOf, progressText } from '../lib/format';
  import type { Title } from '../lib/types';
  import Icon from './Icon.svelte';

  let {
    title,
    color,
    onopen,
  }: { title: Title; color: string; onopen: (title: Title) => void } = $props();

  const LONG_PRESS_MS = 450;
  const MOVE_TOLERANCE = 10;

  let broken = $state(false);
  $effect(() => {
    void title.cover;
    broken = false;
  });

  const blurred = $derived(app.blurAdult && title.library === 'hentailib');
  const percent = $derived(percentOf(title.done_main, title.total_main));
  const complete = $derived(isComplete(title.done_main, title.total_main));
  const unit = $derived(title.medium === 'anime' ? 'серию' : 'главу');
  const picked = $derived(app.selected.has(title.id));

  async function step(delta: -1 | 1) {
    try {
      await app.step(title.id, 'main', delta);
    } catch (e) {
      app.showToast(errorText(e));
    }
  }

  // ---------- долгое нажатие включает режим выбора ----------
  let timer: ReturnType<typeof setTimeout> | undefined;
  let fired = false;
  let startX = 0;
  let startY = 0;

  function cancelPress() {
    clearTimeout(timer);
    timer = undefined;
  }

  function pressDown(e: PointerEvent) {
    fired = false;
    if (app.selecting) return;
    startX = e.clientX;
    startY = e.clientY;
    timer = setTimeout(() => {
      timer = undefined;
      fired = true;
      app.beginSelect(title.id);
    }, LONG_PRESS_MS);
  }

  function pressMove(e: PointerEvent) {
    // палец поехал: это прокрутка, а не долгое нажатие
    if (timer && Math.hypot(e.clientX - startX, e.clientY - startY) > MOVE_TOLERANCE) cancelPress();
  }

  function click() {
    if (fired) {
      fired = false; // клик после долгого нажатия уже обработан
      return;
    }
    if (app.selecting) app.toggleSelect(title.id);
    else onopen(title);
  }
</script>

<article class="card" class:picked style="--c: {color}">
  <button
    type="button"
    class="open"
    aria-pressed={app.selecting ? picked : undefined}
    onclick={click}
    onpointerdown={pressDown}
    onpointermove={pressMove}
    onpointerup={cancelPress}
    onpointercancel={cancelPress}
    onpointerleave={cancelPress}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <span class="cover" class:blurred>
      {#if title.cover && !broken}
        <img src={title.cover} alt="" loading="lazy" decoding="async" draggable="false" onerror={() => (broken = true)} />
      {:else}
        <span class="ph">{title.name.slice(0, 1).toUpperCase()}</span>
      {/if}
    </span>
    <span class="info">
      <span class="name">{title.name}</span>
      <span class="meta">
        <span class="lib" data-lib={title.library}>{LIBRARY_LABEL[title.library]}</span>
        {#key `${title.done_main}:${title.done_volumes}`}
          <span class="progress" in:fly|global={{ y: -6, duration: 160 }}>{progressText(title)}</span>
        {/key}
      </span>
      {#if percent !== null}
        <span class="bar"><span class="fill" style="width: {percent}%"></span></span>
      {/if}
    </span>
  </button>

  {#if app.selecting}
    <span class="check-wrap" aria-hidden="true">
      <span class="check" class:on={picked}>
        {#if picked}<Icon name="check" size={18} />{/if}
      </span>
    </span>
  {:else}
    <div class="steps">
      <button
        type="button"
        class="icon-btn"
        aria-label="Минус одна {unit}"
        disabled={title.done_main === 0}
        onclick={() => step(-1)}
      >
        <Icon name="minus" size={20} />
      </button>
      <button
        type="button"
        class="plus"
        class:complete
        aria-label="Плюс одна {unit}"
        onclick={() => step(1)}
      >
        <Icon name="plus" size={24} />
      </button>
    </div>
  {/if}
</article>

<style>
  .card {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 8px 8px 8px 0;
    background: var(--surface-2);
    border: 1px solid var(--line);
    border-left: 4px solid var(--c);
    border-radius: 14px;
    content-visibility: auto;
    contain-intrinsic-size: auto 92px;
    transition: background 0.15s;
  }
  .card.picked {
    background: var(--surface-2);
    background: color-mix(in srgb, var(--accent) 14%, var(--surface-2));
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .open {
    display: flex;
    flex: 1;
    gap: 12px;
    align-items: center;
    min-width: 0;
    padding-left: 8px;
    text-align: left;
    -webkit-touch-callout: none;
  }

  .cover {
    display: grid;
    flex: none;
    place-items: center;
    width: 52px;
    height: 74px;
    overflow: hidden;
    background: var(--bg);
    border-radius: 8px;
  }
  .cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    pointer-events: none;
  }
  .cover.blurred img {
    filter: blur(10px);
    transform: scale(1.15);
  }
  .ph {
    color: var(--muted);
    font-size: 1.4rem;
    font-weight: 700;
  }

  .info {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .name {
    display: -webkit-box;
    overflow: hidden;
    font-weight: 650;
    line-height: 1.25;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 8px;
    align-items: center;
    color: var(--muted);
    font-size: 0.85rem;
  }

  .progress {
    font-variant-numeric: tabular-nums;
  }

  .lib {
    padding: 1px 8px;
    color: var(--muted);
    border: 1px solid var(--line);
    border-radius: 999px;
    font-size: 0.75rem;
    font-weight: 650;
  }
  .lib[data-lib='mangalib'] {
    color: #8fb8ff;
    border-color: #8fb8ff55;
  }
  .lib[data-lib='hentailib'] {
    color: #ff93d2;
    border-color: #ff93d255;
  }
  .lib[data-lib='animelib'] {
    color: #6fe0d0;
    border-color: #6fe0d055;
  }

  .bar {
    display: block;
    height: 4px;
    overflow: hidden;
    background: var(--bg);
    border-radius: 2px;
  }
  .fill {
    display: block;
    height: 100%;
    background: var(--c);
    border-radius: 2px;
    transition: width 0.25s;
  }

  .steps {
    display: flex;
    flex: none;
    gap: 2px;
    align-items: center;
  }

  .icon-btn:disabled {
    opacity: 0.3;
  }

  .plus {
    display: grid;
    place-items: center;
    width: 52px;
    height: 52px;
    color: #1a1230;
    background: var(--accent);
    border-radius: 50%;
    transition:
      transform 0.12s,
      opacity 0.2s;
  }
  .plus:active {
    transform: scale(0.9);
  }
  .plus.complete {
    opacity: 0.4;
  }

  .check-wrap {
    display: grid;
    flex: none;
    place-items: center;
    width: 54px;
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
  .check.on {
    background: var(--accent);
    border-color: var(--accent);
  }
</style>
