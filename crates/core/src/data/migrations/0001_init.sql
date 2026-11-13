-- Схема версии 1. Новые изменения схемы добавляются новыми файлами 0002_..., 0003_...
-- и никогда не правят уже выпущенные.

CREATE TABLE groups (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    medium   TEXT    NOT NULL CHECK (medium IN ('manga', 'anime')),
    name     TEXT    NOT NULL,
    color    TEXT    NOT NULL,
    position INTEGER NOT NULL
);
CREATE INDEX idx_groups_medium_position ON groups (medium, position);

CREATE TABLE titles (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    medium              TEXT    NOT NULL CHECK (medium IN ('manga', 'anime')),
    kind                TEXT    CHECK (kind IN ('manga', 'manhwa', 'manhua')),
    library             TEXT    NOT NULL CHECK (library IN ('mangalib', 'hentailib', 'animelib', 'other')),
    group_id            INTEGER REFERENCES groups (id) ON DELETE SET NULL,
    name                TEXT    NOT NULL,
    original_name       TEXT,
    cover               TEXT,
    url                 TEXT,
    note                TEXT    NOT NULL DEFAULT '',
    total_main          INTEGER CHECK (total_main IS NULL OR total_main > 0),
    total_volumes       INTEGER CHECK (total_volumes IS NULL OR total_volumes > 0),
    done_main           INTEGER NOT NULL DEFAULT 0 CHECK (done_main >= 0),
    done_volumes        INTEGER NOT NULL DEFAULT 0 CHECK (done_volumes >= 0),
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL,
    progress_updated_at INTEGER NOT NULL,
    library_changed_at  INTEGER
);
CREATE INDEX idx_titles_medium_group ON titles (medium, group_id);

-- Стартовые группы при первом запуске
INSERT INTO groups (medium, name, color, position) VALUES
    ('manga', 'Читаю',       '#6ea8ff', 0),
    ('manga', 'Брошено',     '#9a9ab5', 1),
    ('manga', 'В планах',    '#ffc46b', 2),
    ('manga', 'Прочитано',   '#7ddba8', 3),
    ('manga', 'Любимые',     '#ff8fa3', 4),
    ('anime', 'Смотрю',      '#6ea8ff', 0),
    ('anime', 'Брошено',     '#9a9ab5', 1),
    ('anime', 'В планах',    '#ffc46b', 2),
    ('anime', 'Просмотрено', '#7ddba8', 3),
    ('anime', 'Любимые',     '#ff8fa3', 4);
