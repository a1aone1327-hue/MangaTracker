// Подписи и небольшие вычисления только для отображения. Правила (границы прогресса,
// что можно сохранить) проверяет Rust, здесь они не дублируются.
import type { Counter, Library, MangaKind, Medium, Title, TitleDraft } from './types';

export const LIBRARY_LABEL: Record<Library, string> = {
  mangalib: 'MangaLib',
  hentailib: 'HentaiLib',
  animelib: 'AnimeLib',
  other: 'Другое',
};

export const LIBRARY_ORDER: Library[] = ['mangalib', 'hentailib', 'animelib', 'other'];

export const KIND_LABEL: Record<MangaKind, string> = {
  manga: 'Манга',
  manhwa: 'Манхва',
  manhua: 'Маньхуа',
};

export const KIND_ORDER: MangaKind[] = ['manga', 'manhwa', 'manhua'];

/** Какие библиотеки предлагать в разделе. */
export function librariesFor(medium: Medium): Library[] {
  return medium === 'anime' ? ['animelib', 'other'] : ['mangalib', 'hentailib', 'other'];
}

export function counterLabel(medium: Medium, counter: Counter): string {
  if (counter === 'volumes') return 'Тома';
  return medium === 'anime' ? 'Серии' : 'Главы';
}

export function percentOf(done: number, total: number | null): number | null {
  if (!total) return null;
  return Math.min(100, Math.floor((done * 100) / total));
}

export function isComplete(done: number, total: number | null): boolean {
  return total !== null && total > 0 && done >= total;
}

type ProgressFields = Pick<
  Title,
  'medium' | 'done_main' | 'total_main' | 'done_volumes' | 'total_volumes'
>;

/** «гл. 12/50 · т. 3/8» или «сер. 5/12». */
export function progressText(t: ProgressFields): string {
  const count = (done: number, total: number | null) => (total ? `${done}/${total}` : `${done}`);
  if (t.medium === 'anime') return `сер. ${count(t.done_main, t.total_main)}`;
  const parts = [`гл. ${count(t.done_main, t.total_main)}`];
  if (t.total_volumes || t.done_volumes > 0) {
    parts.push(`т. ${count(t.done_volumes, t.total_volumes)}`);
  }
  return parts.join(' · ');
}

export function toDraft(t: Title): TitleDraft {
  return {
    medium: t.medium,
    kind: t.kind,
    library: t.library,
    group_id: t.group_id,
    name: t.name,
    original_name: t.original_name,
    cover: t.cover,
    url: t.url,
    note: t.note,
    total_main: t.total_main,
    total_volumes: t.total_volumes,
    done_main: t.done_main,
    done_volumes: t.done_volumes,
  };
}

export function newDraft(medium: Medium, groupId: number | null): TitleDraft {
  return {
    medium,
    kind: null,
    library: medium === 'anime' ? 'animelib' : 'mangalib',
    group_id: groupId,
    name: '',
    original_name: null,
    cover: null,
    url: null,
    note: '',
    total_main: null,
    total_volumes: null,
    done_main: 0,
    done_volumes: 0,
  };
}

/** Разбор числа из поля ввода: пусто или мусор значит «нет значения». */
export function parseCount(raw: string): number | null {
  const text = raw.trim();
  if (text === '') return null;
  const n = Math.floor(Number(text));
  return Number.isFinite(n) && n >= 0 ? n : null;
}

export function formatDate(unixSeconds: number): string {
  return new Date(unixSeconds * 1000).toLocaleDateString('ru-RU');
}
