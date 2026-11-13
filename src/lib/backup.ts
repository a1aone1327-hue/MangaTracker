// Вспомогательная логика копий данных: имя файла и быстрый разбор файла для предпросмотра.
// Сам импорт и все проверки делает Rust; здесь только то, что нужно показать до выбора режима.

export interface BackupSummary {
  groups: number;
  manga: number;
  anime: number;
}

/** Сколько всего в файле. `null`, если это не копия Manga Tracker. */
export function summarizeBackup(json: string): BackupSummary | null {
  let data: unknown;
  try {
    data = JSON.parse(json);
  } catch {
    return null;
  }
  if (typeof data !== 'object' || data === null) return null;
  const d = data as { format?: unknown; groups?: unknown; titles?: unknown };
  if (d.format !== 'manga-tracker-backup') return null;

  const groups = Array.isArray(d.groups) ? d.groups.length : 0;
  const titles: unknown[] = Array.isArray(d.titles) ? d.titles : [];
  let anime = 0;
  for (const t of titles) {
    if ((t as { medium?: unknown } | null)?.medium === 'anime') anime += 1;
  }
  return { groups, manga: titles.length - anime, anime };
}

/** `manga-tracker-2026-11-12.json` (по местной дате). */
export function exportFileName(date: Date): string {
  const two = (n: number) => String(n).padStart(2, '0');
  return `manga-tracker-${date.getFullYear()}-${two(date.getMonth() + 1)}-${two(date.getDate())}.json`;
}
