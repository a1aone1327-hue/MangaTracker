// Единственное место, где UI разговаривает с Rust.
// Компоненты не вызывают invoke напрямую: так границу слоёв видно в одном файле.
import { invoke } from '@tauri-apps/api/core';
import { shareFile } from '@choochmeque/tauri-plugin-sharekit-api';
import { openUrl } from '@tauri-apps/plugin-opener';
import type {
  Counter,
  FetchedTitle,
  Group,
  ImportMode,
  ImportReport,
  Library,
  Medium,
  OnDelete,
  Title,
  TitleDraft,
} from './types';

// ---------- группы ----------

export const listGroups = (medium: Medium) => invoke<Group[]>('list_groups', { medium });

export const createGroup = (medium: Medium, name: string, color: string) =>
  invoke<Group>('create_group', { medium, name, color });

export const updateGroup = (id: number, name: string, color: string) =>
  invoke<Group>('update_group', { id, name, color });

export const reorderGroups = (medium: Medium, orderedIds: number[]) =>
  invoke<void>('reorder_groups', { medium, orderedIds });

/** Возвращает, сколько тайтлов затронуто. */
export const deleteGroup = (id: number, onDelete: OnDelete) =>
  invoke<number>('delete_group', { id, onDelete });

// ---------- тайтлы ----------

export const listTitles = (medium: Medium) => invoke<Title[]>('list_titles', { medium });

export const createTitle = (draft: TitleDraft) => invoke<Title>('create_title', { draft });

export const updateTitle = (id: number, draft: TitleDraft) =>
  invoke<Title>('update_title', { id, draft });

export const stepProgress = (id: number, counter: Counter, delta: number) =>
  invoke<Title>('step_progress', { id, counter, delta });

/** `groupId = null` значит «без группы». */
export const moveTitles = (ids: number[], groupId: number | null) =>
  invoke<number>('move_titles', { ids, groupId });

export const setLibrary = (ids: number[], library: Library) =>
  invoke<number>('set_library', { ids, library });

export const deleteTitles = (ids: number[]) => invoke<number>('delete_titles', { ids });

/** Библиотека по ссылке (или null, если ссылка незнакомая). */
export const detectLibrary = (url: string) => invoke<Library | null>('detect_library', { url });

// ---------- данные с сайтов ----------

/** Пробует получить данные по ссылке. `null`, если не вышло (тихая неудача, не ошибка). */
export const fetchTitleInfo = (url: string) =>
  invoke<FetchedTitle | null>('fetch_title_info', { url });

// ---------- копии данных ----------

/** Собирает копию всей базы в файл и возвращает путь к нему. */
export const exportBackup = (fileName: string) => invoke<string>('export_backup', { fileName });

/** Показывает системное меню «Поделиться» с файлом копии. */
export async function shareBackupFile(path: string): Promise<void> {
  try {
    await shareFile(`file://${path}`, {
      mimeType: 'application/json',
      title: 'Копия Manga Tracker',
    });
  } catch (e) {
    // Android сообщает «отменено» даже после успешной отправки, это не ошибка
    if (/cancel/i.test(errorText(e))) return;
    throw e;
  }
}

export const importBackup = (json: string, mode: ImportMode) =>
  invoke<ImportReport>('import_backup', { json, mode });

// ---------- прочее ----------

export const openLink = (url: string) => openUrl(url);

/** Ошибки из Rust приходят строкой с готовым текстом для пользователя. */
export function errorText(e: unknown): string {
  if (typeof e === 'string') return e;
  if (e instanceof Error) return e.message;
  return 'Что-то пошло не так';
}
