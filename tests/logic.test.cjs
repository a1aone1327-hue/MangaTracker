// Проверки чистой логики интерфейса (поиск и сортировка). Запуск: npm run test:logic
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

// скомпилированные файлы лежат рядом, но проект помечен как ESM, а они в формате CommonJS
const dir = path.join(__dirname, '..', '.logic-test');
fs.writeFileSync(path.join(dir, 'package.json'), '{"type":"commonjs"}');
const q = require(path.join(dir, 'query'));
const b = require(path.join(dir, 'backup'));

const t = (id, name, o = {}) => ({
  id, medium: 'manga', kind: null, library: 'mangalib', group_id: null, name,
  original_name: null, cover: null, url: null, note: '', total_main: null, total_volumes: null,
  done_main: 0, done_volumes: 0, created_at: id, updated_at: id, progress_updated_at: id,
  library_changed_at: null, ...o,
});
let passed = 0;
const test = (name, fn) => { fn(); passed++; console.log('ok  -', name); };

test('normalize: регистр, ё, диакритика, й сохраняется', () => {
  assert.equal(q.normalizeText('  Ёжик   В  Тумане '), 'ежик в тумане');
  assert.equal(q.normalizeText('Ōkami'), 'okami');
  assert.equal(q.normalizeText('Мой'), 'мой');
  assert.notEqual(q.normalizeText('Мой'), q.normalizeText('Мои'));
});

test('parseQuery: пустой запрос и слова', () => {
  assert.deepEqual(q.parseQuery('   '), []);
  assert.deepEqual(q.parseQuery(' Соло  Левелинг '), ['соло', 'левелинг']);
});

test('matchesQuery: все слова, оригинальное название, подстрока', () => {
  const a = t(1, 'Поднятие уровня в одиночку', { original_name: 'Solo Leveling' });
  assert.ok(q.matchesQuery(a, q.parseQuery('одиноч')));
  assert.ok(q.matchesQuery(a, q.parseQuery('solo level')));
  assert.ok(q.matchesQuery(a, q.parseQuery('уровня solo')));
  assert.ok(!q.matchesQuery(a, q.parseQuery('берсерк')));
  assert.ok(q.matchesQuery(a, []));
});

test('matchesQuery: ё находится по е', () => {
  assert.ok(q.matchesQuery(t(1, 'Ёжик'), q.parseQuery('ежик')));
  assert.ok(q.matchesQuery(t(2, 'Ежик'), q.parseQuery('ёжик')));
});

test('sort name: числа по возрастанию, регистр не важен', () => {
  const list = [t(1, 'Глава 10'), t(2, 'глава 2'), t(3, 'Арка')];
  const names = q.sortTitles(list, 'name', new Map()).map((x) => x.name);
  assert.deepEqual(names, ['Арка', 'глава 2', 'Глава 10']);
});

test('sort added: новые сверху', () => {
  const list = [t(1, 'A'), t(2, 'B'), t(3, 'C')];
  assert.deepEqual(q.sortTitles(list, 'added', new Map()).map((x) => x.id), [3, 2, 1]);
});

test('sort updated: по снимку, тапы не двигают порядок', () => {
  const list = [t(1, 'A', { progress_updated_at: 100 }), t(2, 'B', { progress_updated_at: 300 }), t(3, 'C', { progress_updated_at: 200 })];
  const ranks = q.buildRanks(list, 'updated');
  assert.deepEqual(q.sortTitles(list, 'updated', ranks).map((x) => x.id), [2, 3, 1]);
  const after = list.map((x) => (x.id === 1 ? { ...x, progress_updated_at: 999 } : x));
  assert.deepEqual(q.sortTitles(after, 'updated', ranks).map((x) => x.id), [2, 3, 1]);
  assert.deepEqual(q.sortTitles(after, 'updated', q.buildRanks(after, 'updated')).map((x) => x.id), [1, 2, 3]);
});

test('sort updated: новый тайтл вне снимка идёт первым', () => {
  const old = [t(1, 'A', { progress_updated_at: 100 }), t(2, 'B', { progress_updated_at: 200 })];
  const ranks = q.buildRanks(old, 'updated');
  const withNew = [...old, t(3, 'C', { progress_updated_at: 50 })];
  assert.equal(q.sortTitles(withNew, 'updated', ranks)[0].id, 3);
});

test('sort progress: процент по убыванию, без «всего» после, законченное внизу', () => {
  const list = [
    t(1, 'Half', { total_main: 10, done_main: 5 }),
    t(2, 'Ongoing', { done_main: 40 }),
    t(3, 'Done', { total_main: 10, done_main: 10 }),
    t(4, 'Almost', { total_main: 10, done_main: 9 }),
  ];
  const ranks = q.buildRanks(list, 'progress');
  assert.deepEqual(q.sortTitles(list, 'progress', ranks).map((x) => x.name), ['Almost', 'Half', 'Ongoing', 'Done']);
});

test('volatile: только updated и progress; подписи есть у всех режимов', () => {
  assert.deepEqual(q.VOLATILE_SORTS.slice().sort(), ['progress', 'updated']);
  for (const m of q.SORT_MODES) { assert.ok(q.SORT_LABEL[m]); assert.ok(q.SORT_HINT[m]); }
});

test('backup: имя файла по местной дате', () => {
  assert.equal(b.exportFileName(new Date(2026, 10, 3)), 'manga-tracker-2026-11-03.json');
  assert.equal(b.exportFileName(new Date(2027, 0, 9)), 'manga-tracker-2027-01-09.json');
});

test('backup: предпросмотр считает группы и разделы', () => {
  const json = JSON.stringify({
    format: 'manga-tracker-backup', version: 1, exported_at: 0,
    groups: [{ id: 1 }, { id: 2 }],
    titles: [{ medium: 'manga' }, { medium: 'manga' }, { medium: 'anime' }],
  });
  assert.deepEqual(b.summarizeBackup(json), { groups: 2, manga: 2, anime: 1 });
});

test('backup: чужой, битый и пустой файл не принимаются', () => {
  assert.equal(b.summarizeBackup('{ не json'), null);
  assert.equal(b.summarizeBackup('[]'), null);
  assert.equal(b.summarizeBackup('null'), null);
  assert.equal(b.summarizeBackup(JSON.stringify({ format: 'other' })), null);
  assert.deepEqual(b.summarizeBackup(JSON.stringify({ format: 'manga-tracker-backup' })), { groups: 0, manga: 0, anime: 0 });
});

console.log(`\n${passed} tests passed`);
