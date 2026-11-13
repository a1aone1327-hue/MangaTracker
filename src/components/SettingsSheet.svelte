<script lang="ts">
  import { errorText, exportBackup, shareBackupFile } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { exportFileName } from '../lib/backup';
  import Sheet from './Sheet.svelte';

  let {
    onclose,
    onimport,
  }: { onclose: () => void; onimport: (fileName: string, json: string) => void } = $props();

  const MAX_IMPORT_BYTES = 200 * 1024 * 1024;

  let picker: HTMLInputElement | undefined = $state();
  let exporting = $state(false);

  async function exportData() {
    if (exporting) return;
    exporting = true;
    try {
      await app.settle(); // отложенные удаления применяем, чтобы копия была актуальной
      const path = await exportBackup(exportFileName(new Date()));
      await shareBackupFile(path);
    } catch (e) {
      app.showToast(errorText(e));
    } finally {
      exporting = false;
    }
  }

  async function picked(e: Event & { currentTarget: HTMLInputElement }) {
    const file = e.currentTarget.files?.[0];
    e.currentTarget.value = '';
    if (!file) return;
    if (file.size > MAX_IMPORT_BYTES) {
      app.showToast('Файл слишком большой');
      return;
    }
    try {
      onimport(file.name, await file.text());
    } catch {
      app.showToast('Не удалось прочитать файл');
    }
  }
</script>

<Sheet title="Настройки" {onclose}>
  <button
    type="button"
    class="row"
    role="switch"
    aria-checked={app.blurAdult}
    onclick={() => app.setBlurAdult(!app.blurAdult)}
  >
    <span class="text">
      <strong>Размывать обложки HentaiLib</strong>
      <small>Чтобы телефон можно было показать кому угодно</small>
    </span>
    <span class="track" class:on={app.blurAdult}><span class="thumb"></span></span>
  </button>

  <h3>Данные</h3>
  <p class="about">
    Копия — обычный JSON-файл со всеми группами и тайтлами обоих разделов. Сохрани его в облако или
    отправь себе, чтобы перенести данные на новый телефон.
  </p>

  <button type="button" class="row" disabled={exporting} onclick={exportData}>
    <span class="text">
      <strong>{exporting ? 'Готовлю файл…' : 'Экспорт копии'}</strong>
      <small>Откроется меню «Поделиться»</small>
    </span>
  </button>

  <button type="button" class="row" onclick={() => picker?.click()}>
    <span class="text">
      <strong>Импорт из файла</strong>
      <small>Слить с текущими данными или заменить всё</small>
    </span>
  </button>
  <input
    bind:this={picker}
    class="file"
    type="file"
    accept=".json,application/json"
    tabindex="-1"
    aria-hidden="true"
    onchange={picked}
  />
</Sheet>

<style>
  .row {
    display: flex;
    gap: 16px;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    min-height: 64px;
    padding: 8px 0;
    text-align: left;
  }
  .row:disabled {
    opacity: 0.5;
  }

  .text strong,
  .text small {
    display: block;
  }
  .text small {
    margin-top: 2px;
    color: var(--muted);
  }

  h3 {
    margin: 22px 0 6px;
    padding-top: 18px;
    border-top: 1px solid var(--line);
    font-size: 1rem;
  }

  .about {
    margin: 0 0 6px;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .file {
    position: absolute;
    width: 1px;
    height: 1px;
    opacity: 0;
    pointer-events: none;
  }

  .track {
    position: relative;
    flex: none;
    width: 52px;
    height: 30px;
    background: var(--surface-2);
    border: 1px solid var(--line);
    border-radius: 999px;
    transition: background 0.2s;
  }
  .track.on {
    background: var(--accent);
    border-color: transparent;
  }

  .thumb {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 22px;
    height: 22px;
    background: var(--text);
    border-radius: 50%;
    transition: transform 0.2s;
  }
  .track.on .thumb {
    background: #1a1230;
    transform: translateX(22px);
  }
</style>
