// Поиск и сортировка списка. Чистые функции без состояния, чтобы их можно было проверять отдельно.
import { isComplete, percentOf } from './format';
import type { Title } from './types';

export type SortMode = 'updated' | 'name' | 'added' | 'progress';

export const SORT_MODES: SortMode[] = ['updated', 'name', 'added', 'progress'];

export const SORT_LABEL: Record<SortMode, string> = {
  updated: 'Недавно обновлённые',
  name: 'По названию',
  added: 'Недавно добавленные',
  progress: 'По прогрессу',
};

export const SORT_HINT: Record<SortMode, string> = {
  updated: 'Сверху то, где недавно менялся прогресс',
  name: 'От А до Я, числа по возрастанию',
  added: 'Сверху самые новые',
  progress: 'Сверху то, что ближе к концу, законченное внизу',
};

/** Эти порядки зависят от прогресса, поэтому пересчитываются не на каждый тап (см. buildRanks). */
export const VOLATILE_SORTS: SortMode[] = ['updated', 'progress'];

const collator = new Intl.Collator('ru', { numeric: true, sensitivity: 'base' });
const byName = (a: Title, b: Title) => collator.compare(a.name, b.name) || a.id - b.id;

/**
 * Приводит текст к виду для поиска: без регистра, «ё» как «е», без значков над латиницей
 * (Ō как O). Кириллическое «й» остаётся «й».
 */
export function normalizeText(text: string): string {
  return text
    .normalize('NFKD')
    .replace(/([A-Za-z])\p{M}+/gu, '$1')
    .normalize('NFC')
    .toLowerCase()
    .replace(/ё/g, 'е')
    .replace(/\s+/g, ' ')
    .trim();
}

/** Запрос превращается в слова: тайтл подходит, если в нём есть все слова. */
export function parseQuery(query: string): string[] {
  const normalized = normalizeText(query);
  return normalized === '' ? [] : normalized.split(' ');
}

export function matchesQuery(
  title: Pick<Title, 'name' | 'original_name'>,
  tokens: readonly string[],
): boolean {
  if (tokens.length === 0) return true;
  const haystack = normalizeText(`${title.name} ${title.original_name ?? ''}`);
  return tokens.every((token) => haystack.includes(token));
}

function volatileCompare(mode: SortMode): (a: Title, b: Title) => number {
  if (mode === 'progress') {
    // в процессе по убыванию процента, затем «всего неизвестно», законченное в самом низу
    const key = (t: Title) =>
      isComplete(t.done_main, t.total_main) ? -2 : (percentOf(t.done_main, t.total_main) ?? -1);
    return (a, b) => key(b) - key(a) || b.done_main - a.done_main || byName(a, b);
  }
  return (a, b) => b.progress_updated_at - a.progress_updated_at || byName(a, b);
}

/**
 * Снимок порядка для «живых» сортировок. Пока ты жмёшь +1, порядок не пересчитывается,
 * иначе карточки прыгали бы из-под пальца. Пересчёт: при выборе сортировки, загрузке
 * данных и возвращении в приложение.
 */
export function buildRanks(titles: readonly Title[], mode: SortMode): Map<number, number> {
  const sorted = [...titles].sort(volatileCompare(mode));
  return new Map(sorted.map((t, i) => [t.id, i]));
}

/** Сортирует тайтлы. Новые, которых нет в снимке, идут первыми. */
export function sortTitles(
  list: readonly Title[],
  mode: SortMode,
  ranks: ReadonlyMap<number, number>,
): Title[] {
  const items = [...list];
  if (mode === 'name') return items.sort(byName);
  if (mode === 'added') return items.sort((a, b) => b.created_at - a.created_at || b.id - a.id);
  return items.sort((a, b) => (ranks.get(a.id) ?? -1) - (ranks.get(b.id) ?? -1) || byName(a, b));
}
