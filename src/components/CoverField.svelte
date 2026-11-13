<script lang="ts">
  import { fileToThumbnail } from '../lib/image';
  import Icon from './Icon.svelte';

  let {
    cover = $bindable(),
    blurred = false,
    oncommit,
  }: { cover: string | null; blurred?: boolean; oncommit?: () => void } = $props();

  let picker: HTMLInputElement | undefined = $state();
  let error = $state('');
  let broken = $state(false);

  const isLocal = $derived(cover?.startsWith('data:') ?? false);

  $effect(() => {
    void cover;
    broken = false;
  });

  async function pick(e: Event & { currentTarget: HTMLInputElement }) {
    const file = e.currentTarget.files?.[0];
    e.currentTarget.value = '';
    if (!file) return;
    try {
      cover = await fileToThumbnail(file);
      error = '';
      oncommit?.();
    } catch {
      error = 'Не удалось прочитать изображение';
    }
  }

  function clear() {
    cover = null;
    error = '';
    oncommit?.();
  }
</script>

<div class="cover-field">
  <div class="preview" class:blurred>
    {#if cover && !broken}
      <img src={cover} alt="Обложка" onerror={() => (broken = true)} />
    {:else}
      <Icon name="image" size={28} />
    {/if}
  </div>

  <div class="controls">
    <input
      bind:this={picker}
      class="file"
      type="file"
      accept="image/*"
      onchange={pick}
      tabindex="-1"
      aria-hidden="true"
    />
    <button type="button" class="btn" onclick={() => picker?.click()}>Выбрать фото</button>
    <input
      class="input"
      inputmode="url"
      autocomplete="off"
      placeholder="или ссылка на обложку"
      value={isLocal ? '' : (cover ?? '')}
      oninput={(e) => (cover = e.currentTarget.value)}
      onchange={() => oncommit?.()}
    />
    {#if cover}
      <button type="button" class="link" onclick={clear}>Убрать обложку</button>
    {/if}
    {#if error}<p class="error" role="alert">{error}</p>{/if}
  </div>
</div>

<style>
  .cover-field {
    display: flex;
    gap: 14px;
    align-items: flex-start;
  }

  .preview {
    display: grid;
    flex: none;
    place-items: center;
    width: 84px;
    height: 120px;
    overflow: hidden;
    color: var(--muted);
    background: var(--bg);
    border: 1px solid var(--line);
    border-radius: 12px;
  }
  .preview img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .preview.blurred img {
    filter: blur(10px);
    transform: scale(1.15);
  }

  .controls {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
  }

  .file {
    position: absolute;
    width: 1px;
    height: 1px;
    opacity: 0;
    pointer-events: none;
  }

  .link {
    align-self: flex-start;
    min-height: 36px;
    color: var(--muted);
    text-decoration: underline;
  }

  .error {
    margin: 0;
    color: var(--danger);
  }
</style>
