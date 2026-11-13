<script lang="ts">
  import { onMount } from 'svelte';
  import { errorText } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { newDraft } from '../lib/format';
  import type { TitleDraft } from '../lib/types';
  import Sheet from './Sheet.svelte';
  import TitleFields from './TitleFields.svelte';

  let { onclose }: { onclose: () => void } = $props();

  let draft = $state<TitleDraft>(newDraft(app.medium, app.defaultGroupId()));
  let fields: { focusName: () => void } | undefined = $state();
  let busy = $state(false);
  let error = $state('');

  onMount(() => fields?.focusName());

  // more = true: оставить шторку открытой для следующего тайтла (удобно переносить список из заметок)
  async function add(more: boolean) {
    if (busy) return;
    busy = true;
    error = '';
    try {
      const created = await app.createTitle($state.snapshot(draft));
      if (more) {
        // группа, библиотека и тип сохраняются, остальное пустое
        const next = newDraft(app.medium, created.group_id);
        next.library = created.library;
        next.kind = created.kind;
        draft = next;
        app.showToast(`Добавлено: ${created.name}`, 1800);
        fields?.focusName();
      } else {
        onclose();
      }
    } catch (e) {
      error = errorText(e);
    } finally {
      busy = false;
    }
  }
</script>

<Sheet title="Новый тайтл" {onclose}>
  <TitleFields bind:this={fields} bind:draft />

  {#if error}<p class="error" role="alert">{error}</p>{/if}

  <div class="actions">
    <button type="button" class="btn primary" disabled={busy || !draft.name.trim()} onclick={() => add(false)}>
      Добавить
    </button>
    <button type="button" class="btn" disabled={busy || !draft.name.trim()} onclick={() => add(true)}>
      Добавить и ещё
    </button>
  </div>
</Sheet>

<style>
  .actions {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .error {
    margin: 0 0 14px;
    color: var(--danger);
  }
</style>
