// Состояние приложения. Данные берём из Rust через api.ts, правила проверяет ядро.
import { SvelteSet } from 'svelte/reactivity';
import * as api from './api';
import { pushBack } from './back';
import { SORT_MODES, VOLATILE_SORTS, buildRanks, matchesQuery, parseQuery, sortTitles, type SortMode } from './query';
import { LIBRARY_ORDER } from './format';
import type {
  Counter,
  Group,
  ImportMode,
  ImportReport,
  Library,
  Medium,
  OnDelete,
  Title,
  TitleDraft,
} from './types';

const UNDO_MS = 5000;

export interface ToastState {
  text: string;
  action?: { label: string; run: () => void };
}

function loadMedium(): Medium {
  try {
    return localStorage.getItem('medium') === 'anime' ? 'anime' : 'manga';
  } catch {
    return 'manga';
  }
}

function loadFlag(key: string): boolean {
  try {
    return localStorage.getItem(key) === '1';
  } catch {
    return false;
  }
}

function loadSort(): SortMode {
  try {
    const saved = localStorage.getItem('sort');
    return SORT_MODES.find((m) => m === saved) ?? 'updated';
  } catch {
    return 'updated';
  }
}

function saveSetting(key: string, value: string): void {
  try {
    localStorage.setItem(key, value);
  } catch {
    // хранилище недоступно: настройка просто не запомнится
  }
}

class AppState {
  medium = $state<Medium>(loadMedium());
  groups = $state<Group[]>([]);
  titles = $state<Title[]>([]);
  ready = $state(false);
  failure = $state<string | null>(null);
  toast = $state<ToastState | null>(null);

  libraryFilter = $state<'all' | Library>('all');
  blurAdult = $state(loadFlag('blurAdult'));
  lastGroupId = $state<number | null>(null);

  /** Поиск по названию и порядок списка. */
  query = $state('');
  sort = $state<SortMode>(loadSort());
  private ranks = $state<Map<number, number>>(new Map());

  /** Сколько тайтлов в каждой группе (все, без учёта фильтра). */
  counts = $derived.by(() => {
    const map = new Map<number, number>();
    for (const t of this.titles) {
      if (t.group_id !== null) map.set(t.group_id, (map.get(t.group_id) ?? 0) + 1);
    }
    return map;
  });

  /** Чипы фильтра по библиотеке: только те, где есть тайтлы. */
  libraryChips = $derived.by(() => {
    const map = new Map<Library, number>();
    for (const t of this.titles) map.set(t.library, (map.get(t.library) ?? 0) + 1);
    return LIBRARY_ORDER.filter((l) => map.has(l)).map((l) => ({
      library: l,
      count: map.get(l) ?? 0,
    }));
  });

  /** Фильтр, который реально действует (если библиотека опустела, показываем всё). */
  activeFilter: 'all' | Library = $derived(
    this.libraryFilter !== 'all' && this.libraryChips.some((c) => c.library === this.libraryFilter)
      ? this.libraryFilter
      : 'all',
  );

  /** Слова поискового запроса (пусто, если поиск не задан). */
  tokens = $derived(parseQuery(this.query));

  /** Включён ли поиск или фильтр по библиотеке. */
  isFiltering = $derived(this.tokens.length > 0 || this.activeFilter !== 'all');

  /** Тайтлы, прошедшие фильтр по библиотеке и поиск. */
  visibleTitles = $derived.by(() => {
    const byLibrary =
      this.activeFilter === 'all'
        ? this.titles
        : this.titles.filter((t) => t.library === this.activeFilter);
    return this.tokens.length === 0
      ? byLibrary
      : byLibrary.filter((t) => matchesQuery(t, this.tokens));
  });

  /** Видимые тайтлы по группам (ключ null значит «без группы») в выбранном порядке. */
  visibleByGroup = $derived.by(() => {
    const map = new Map<number | null, Title[]>();
    for (const t of this.visibleTitles) {
      const list = map.get(t.group_id);
      if (list) list.push(t);
      else map.set(t.group_id, [t]);
    }
    for (const [key, list] of map) map.set(key, sortTitles(list, this.sort, this.ranks));
    return map;
  });

  private toastTimer: ReturnType<typeof setTimeout> | undefined;
  private chain: Promise<unknown> = Promise.resolve();
  private pending: { titles: Title[]; timer: ReturnType<typeof setTimeout> } | null = null;
  private releaseBack: (() => void) | undefined;

  /** Режим выбора (массовые операции). */
  selecting = $state(false);
  selected = new SvelteSet<number>();

  /** Вызывается один раз при запуске. */
  init(): void {
    // Если приложение уходит в фон, отложенное удаление применяем сразу
    document.addEventListener('visibilitychange', () => {
      if (document.visibilityState === 'hidden') void this.flushPending();
      // вернулись в приложение: самое время обновить порядок «живых» сортировок
      else this.resort();
    });
  }

  /** Все записи в базу идут по очереди: быстрые тапы не обгоняют друг друга. */
  private enqueue<T>(job: () => Promise<T>): Promise<T> {
    const run = this.chain.then(job);
    this.chain = run.catch(() => undefined);
    return run;
  }

  async load(): Promise<void> {
    const medium = this.medium;
    try {
      const [groups, titles] = await Promise.all([api.listGroups(medium), api.listTitles(medium)]);
      if (medium !== this.medium) return; // пока грузили, раздел уже переключили
      this.groups = groups;
      this.titles = titles;
      this.resort();
      this.failure = null;
    } catch (e) {
      if (medium === this.medium) this.failure = api.errorText(e);
    } finally {
      if (medium === this.medium) this.ready = true;
    }
  }

  async setMedium(medium: Medium): Promise<void> {
    if (medium === this.medium) return;
    this.endSelect();
    await this.flushPending();
    this.medium = medium;
    this.libraryFilter = 'all';
    this.query = '';
    this.ready = false;
    saveSetting('medium', medium);
    await this.load();
  }

  setBlurAdult(value: boolean): void {
    this.blurAdult = value;
    saveSetting('blurAdult', value ? '1' : '0');
  }

  // ---------- порядок и фильтры ----------

  /** Пересчитывает снимок порядка для сортировок, зависящих от прогресса. */
  resort(): void {
    this.ranks = VOLATILE_SORTS.includes(this.sort) ? buildRanks(this.titles, this.sort) : new Map();
  }

  setSort(mode: SortMode): void {
    this.sort = mode;
    saveSetting('sort', mode);
    this.resort();
  }

  resetFilters(): void {
    this.query = '';
    this.libraryFilter = 'all';
  }

  // ---------- группы ----------

  async createGroup(name: string, color: string): Promise<void> {
    await api.createGroup(this.medium, name, color);
    this.groups = await api.listGroups(this.medium);
  }

  async updateGroup(id: number, name: string, color: string): Promise<void> {
    await api.updateGroup(id, name, color);
    this.groups = await api.listGroups(this.medium);
  }

  /** Сдвиг группы на одну позицию. Список обновляем сразу, при ошибке откатываем. */
  async moveGroup(id: number, delta: -1 | 1): Promise<void> {
    const ids = this.groups.map((g) => g.id);
    const i = ids.indexOf(id);
    const j = i + delta;
    if (i < 0 || j < 0 || j >= ids.length) return;
    [ids[i], ids[j]] = [ids[j], ids[i]];
    const previous = this.groups;
    this.groups = ids.map((gid) => previous.find((g) => g.id === gid)!);
    try {
      await api.reorderGroups(this.medium, ids);
    } catch (e) {
      this.groups = previous;
      this.showToast(api.errorText(e));
    }
  }

  /** Возвращает, сколько тайтлов затронуто. Тайтлы перечитываем: у них поменялась группа. */
  async deleteGroup(id: number, onDelete: OnDelete): Promise<number> {
    await this.flushPending();
    const moved = await api.deleteGroup(id, onDelete);
    await this.load();
    return moved;
  }

  /** Группа для нового тайтла: последняя использованная или первая в списке. */
  defaultGroupId(): number | null {
    const last = this.lastGroupId;
    if (last !== null && this.groups.some((g) => g.id === last)) return last;
    return this.groups[0]?.id ?? null;
  }

  // ---------- тайтлы ----------

  private replace(title: Title): void {
    this.titles = this.titles.map((t) => (t.id === title.id ? title : t));
  }

  async createTitle(draft: TitleDraft): Promise<Title> {
    const title = await this.enqueue(() => api.createTitle(draft));
    this.titles = [...this.titles, title];
    this.lastGroupId = title.group_id;
    return title;
  }

  async updateTitle(id: number, draft: TitleDraft): Promise<Title> {
    const title = await this.enqueue(() => api.updateTitle(id, draft));
    this.replace(title);
    return title;
  }

  /** Быстрые кнопки +1/−1. */
  async step(id: number, counter: Counter, delta: -1 | 1): Promise<Title> {
    const title = await this.enqueue(() => api.stepProgress(id, counter, delta));
    this.replace(title);
    return title;
  }

  /**
   * Удаление с откатом: тайтлы сразу пропадают из списка, а из базы удаляются через
   * несколько секунд. Пока идёт время, их можно вернуть кнопкой «Отменить».
   */
  removeTitle(id: number): void {
    this.removeTitles([id]);
  }

  removeTitles(ids: number[]): void {
    const wanted = new Set(ids);
    const gone = this.titles.filter((t) => wanted.has(t.id));
    if (gone.length === 0) return;
    void this.flushPending();
    this.titles = this.titles.filter((t) => !wanted.has(t.id));
    const timer = setTimeout(() => void this.flushPending(), UNDO_MS);
    this.pending = { titles: gone, timer };
    const text = gone.length === 1 ? `«${gone[0].name}» удалён` : `Удалено тайтлов: ${gone.length}`;
    this.showToast(text, UNDO_MS, { label: 'Отменить', run: () => this.undoRemove() });
  }

  private undoRemove(): void {
    const p = this.pending;
    if (!p) return;
    clearTimeout(p.timer);
    this.pending = null;
    const back = p.titles.filter((t) => t.medium === this.medium);
    this.titles = [...this.titles, ...back];
    this.toast = null;
  }

  private async flushPending(): Promise<void> {
    const p = this.pending;
    if (!p) return;
    clearTimeout(p.timer);
    this.pending = null;
    try {
      await this.enqueue(() => api.deleteTitles(p.titles.map((t) => t.id)));
    } catch (e) {
      const back = p.titles.filter((t) => t.medium === this.medium);
      this.titles = [...this.titles, ...back];
      this.showToast(api.errorText(e));
    }
  }

  // ---------- выбор и массовые операции ----------

  /** Включает режим выбора и отмечает тайтл (долгое нажатие). */
  beginSelect(id: number): void {
    if (!this.selecting) {
      this.selecting = true;
      // системная кнопка «назад» сначала выходит из режима выбора
      this.releaseBack = pushBack(() => this.endSelect());
    }
    this.selected.add(id);
  }

  toggleSelect(id: number): void {
    if (this.selected.has(id)) this.selected.delete(id);
    else this.selected.add(id);
  }

  /** Отметить или снять сразу несколько (например, всю группу). */
  selectMany(ids: number[], on: boolean): void {
    for (const id of ids) {
      if (on) this.selected.add(id);
      else this.selected.delete(id);
    }
  }

  selectAllVisible(): void {
    for (const t of this.visibleTitles) this.selected.add(t.id);
  }

  endSelect(): void {
    this.selecting = false;
    this.selected.clear();
    const release = this.releaseBack;
    this.releaseBack = undefined;
    release?.();
  }

  /** Перенос выбранных в группу (null значит «без группы»). Возвращает, сколько перенесено. */
  async moveSelected(groupId: number | null): Promise<number> {
    const ids = [...this.selected];
    const moved = await this.enqueue(() => api.moveTitles(ids, groupId));
    await this.load();
    this.endSelect();
    return moved;
  }

  /** Смена библиотеки у выбранных. Возвращает, у скольких она реально поменялась. */
  async setLibrarySelected(library: Library): Promise<number> {
    const ids = [...this.selected];
    const changed = await this.enqueue(() => api.setLibrary(ids, library));
    await this.load();
    this.endSelect();
    return changed;
  }

  /** Удаление выбранных (подтверждение спрашивает интерфейс). Возвращает их число. */
  deleteSelected(): number {
    const ids = [...this.selected];
    this.removeTitles(ids);
    this.endSelect();
    return ids.length;
  }

  // ---------- копии данных ----------

  /** Применяет отложенные удаления, чтобы экспорт и импорт видели актуальные данные. */
  settle(): Promise<void> {
    return this.flushPending();
  }

  async importBackup(json: string, mode: ImportMode): Promise<ImportReport> {
    this.endSelect();
    await this.flushPending();
    const report = await this.enqueue(() => api.importBackup(json, mode));
    this.resetFilters();
    this.ready = false;
    await this.load();
    return report;
  }

  // ---------- уведомления ----------

  showToast(text: string, ms = 2800, action?: ToastState['action']): void {
    clearTimeout(this.toastTimer);
    this.toast = { text, action };
    this.toastTimer = setTimeout(() => {
      this.toast = null;
    }, ms);
  }
}

export const app = new AppState();
