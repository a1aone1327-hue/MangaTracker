// Зеркало типов ядра (crates/core/src/domain). Названия полей совпадают с JSON из Rust.

export type Medium = 'manga' | 'anime';
export type Library = 'mangalib' | 'hentailib' | 'animelib' | 'other';
export type MangaKind = 'manga' | 'manhwa' | 'manhua';

export interface Group {
  id: number;
  medium: Medium;
  name: string;
  color: string; // #rrggbb
  position: number;
}

export interface Title {
  id: number;
  medium: Medium;
  kind: MangaKind | null;
  library: Library;
  group_id: number | null;
  name: string;
  original_name: string | null;
  cover: string | null;
  url: string | null;
  note: string;
  total_main: number | null;
  total_volumes: number | null;
  done_main: number;
  done_volumes: number;
  created_at: number;
  updated_at: number;
  progress_updated_at: number;
  library_changed_at: number | null;
}

/** Редактируемые поля тайтла (то, что уходит в Rust при создании и сохранении). */
export type TitleDraft = Omit<
  Title,
  'id' | 'created_at' | 'updated_at' | 'progress_updated_at' | 'library_changed_at'
>;

/** Какой счётчик меняем: главы или серии (main) и тома (volumes). */
export type Counter = 'main' | 'volumes';

/** Что сделать с тайтлами удаляемой группы. */
export type OnDelete = { action: 'move_to'; group_id: number } | { action: 'ungroup' };

/** Массовые действия над выбранными тайтлами. */
export type BulkAction = 'move' | 'library' | 'delete';

/** Как импортировать копию: заменить всё или слить с текущими данными. */
export type ImportMode = 'replace' | 'merge';

export interface ImportReport {
  groups_added: number;
  titles_added: number;
  titles_merged: number;
  titles_skipped: number;
}

/** Данные тайтла, найденные на сайте по ссылке. */
export interface FetchedTitle {
  medium: Medium;
  library: Library;
  name: string;
  original_name: string | null;
  total_main: number | null;
  kind: MangaKind | null;
  cover_url: string | null;
  /** Обложка в виде data URL (оригинал, его ещё надо уменьшить). */
  cover_data: string | null;
}
