-- Enums as custom types
CREATE TYPE "Categories" AS ENUM ('Anime', 'Manga', 'Game', 'TV', 'Movie');
CREATE TYPE "Difficulties" AS ENUM ('Easy', 'Medium', 'Hard');

-- `charades` table
CREATE TABLE charades (
    id SERIAL PRIMARY KEY,
    category "Categories" NOT NULL,
    hint TEXT NOT NULL,
    puzzle TEXT NOT NULL,
    solution TEXT NOT NULL,
    difficulty "Difficulties" NOT NULL,
    userid DECIMAL NOT NULL,
    public BOOLEAN NOT NULL
);

-- `discord_settings` table
CREATE TABLE discord_settings (
    id SERIAL PRIMARY KEY,
    server_id BIGINT NOT NULL,
    prefix VARCHAR NOT NULL
);

-- `feeds` table
CREATE TABLE feeds (
    id SERIAL PRIMARY KEY,
    server_id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,
    manga_id TEXT NOT NULL
);

-- `roles` table
CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    server_id BIGINT NOT NULL,
    data TEXT NOT NULL
);
