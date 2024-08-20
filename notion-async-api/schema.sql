CREATE TABLE IF NOT EXISTS blocks (
    -- `rowid` will be the actual primary key, `id` is just a unique index
    id TEXT not null primary key,

    parent_type TEXT not null,
    parent_id TEXT not null,

    created_time TEXT not null,
    created_by TEXT not null,
    last_edited_time TEXT not null,
    last_edited_by TEXT not null,

    -- 1: true, 0: false
    archived INTEGER not null,
    in_trash INTEGER not null,

    -------------------------------
    -- index in parent
    child_index INTEGER not null,

    has_children INTEGER not null,

    -- child_page, child_database, paragraph, etc.
    block_type TEXT not null,
    type_data TEXT not null
);

CREATE TABLE IF NOT EXISTS pages (
    id TEXT not null primary key,

    parent_type TEXT not null,
    parent_id TEXT not null,

    created_time TEXT not null,
    created_by TEXT not null,
    last_edited_time TEXT not null,
    last_edited_by TEXT not null,

    -- 1: true, 0: false
    archived INTEGER not null,
    in_trash INTEGER not null,

    -------------------------------
    -- json object
    properties TEXT not null,
    url TEXT not null,

    public_url TEXT,
    icon TEXT,
    cover TEXT
);

CREATE TABLE IF NOT EXISTS databases (
    id TEXT not null primary key,

    parent_type TEXT not null,
    parent_id TEXT not null,

    created_time TEXT not null,
    created_by TEXT not null,
    last_edited_time TEXT not null,
    last_edited_by TEXT not null,

    -- 1: true, 0: false
    archived INTEGER not null,
    in_trash INTEGER not null,

    -------------------------------
    -- json object
    properties TEXT not null,
    url TEXT not null,

    public_url TEXT,
    icon TEXT,
    cover TEXT,

    is_inline INTEGER not null,
    -- array of rich text objects
    title TEXT not null,
    -- array of rich text objects
    description TEXT not null
);

CREATE TABLE IF NOT EXISTS comments (
    id TEXT not null primary key,

    parent_type TEXT not null,
    parent_id TEXT not null,

    created_time TEXT not null,
    created_by TEXT not null,
    last_edited_time TEXT not null,

    -------------------------------
    discussion_id TEXT not null,
    -- array of rich text objects
    rich_text TEXT not null
);
