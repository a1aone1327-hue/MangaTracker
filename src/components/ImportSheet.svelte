<script lang="ts">
  import { errorText } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { summarizeBackup } from '../lib/backup';
  import type { ImportMode, ImportReport } from '../lib/types';
  import Sheet from './Sheet.svelte';

  let { fileName, json, onclose }: { fileName: string; json: string; onclose: () => void } =
    $props();

  const summary = summarizeBackup(json);

  let step = $state<'choose' | 'confirm'>('choose');
  let busy = $state(false);
  let error = $state('');

  function describe(mode: ImportMode, r: ImportReport): string {
    const skipped = r.titles_skipped > 0 ? `, пропущено: ${r.titles_skipped}` : '';
    if (mode === 'replace') return `Данные заменены. Тайтлов: ${r.titles_added}${skipped}`;
    return `Добавлено: ${r.titles_added}, объединено: ${r.titles_merged}${skipped}`;
  }

  async function run(mode: ImportMode) {
    if (busy) return;
    busy = true;
    error = '';
    try {
      const report = await app.importBackup(json, mode);
      onclose();
      app.showToast(describe(mode, report), 4500);
    } catch (e) {
      error = errorText(e);
      step = 'choose';
    } finally {
      busy = false;
    }
  }
</script>

<Sheet title="Импорт из файла" {onclose}>
  {#if !summary}
    <p class="lead">Файл «{fileName}» не похож на копию Manga Tracker. Выбери файл, который создан экспортом из приложения.</p>
    <button type="button" class="btn" onclick={onclose}>Закрыть</button>
  {:else if step === 'choose'}
    <p class="lead">
      Файл «{fileName}»: манга — {summary.manga}, аниме — {summary.anime}, групп — {summary.groups}.
    </p>

    <button type="button" class="option primary" disabled={busy} onclick={() => run('merge')}>
      <strong>Слить с текущим</strong>
      <small>
        Добавит новые тайтлы и группы. Если тайтл уже есть, возьмёт больший прогресс и дозаполнит
        пустые поля. Ничего не удалит.
      </small>
    </button>

    <button type="button" class="option" disabled={busy} onclick={() => (step = 'confirm')}>
      <strong>Заменить всё</strong>
      <small>Текущие данные будут удалены, вместо них появится содержимое файла.</small>
    </button>

    {#if error}<p class="error" role="alert">{error}</p>{/if}
  {:else}
    <p class="lead">
      Все текущие группы и тайтлы будут удалены и заменены содержимым файла. Это нельзя отменить.
      Если сомневаешься, сначала сделай экспорт текущих данных.
    </p>
    <div class="actions">
      <button type="button" class="btn danger solid" disabled={busy} onclick={() => run('replace')}>
        {busy ? 'Заменяю…' : 'Заменить всё'}
      </button>
      <button type="button" class="btn" disabled={busy} onclick={() => (step = 'choose')}>Назад</button>
    </div>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
  {/if}
</Sheet>

<style>
  .lead {
    margin: 0 0 16px;
    color: var(--muted);
  }

  .option {
    display: block;
    width: 100%;
    margin-bottom: 10px;
    padding: 14px 16px;
    text-align: left;
    background: var(--surface-2);
    border: 1.5px solid var(--line);
    border-radius: 14px;
    transition:
      border-color 0.15s,
      transform 0.12s;
  }
  .option:active {
    transform: scale(0.99);
  }
  .option.primary {
    border-color: var(--accent);
  }
  .option:disabled {
    opacity: 0.5;
  }

  .option strong,
  .option small {
    display: block;
  }
  .option small {
    margin-top: 4px;
    color: var(--muted);
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .error {
    margin: 14px 0 0;
    color: var(--danger);
  }
</style>
