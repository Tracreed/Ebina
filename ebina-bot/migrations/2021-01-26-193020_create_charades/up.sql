CREATE TYPE categories AS ENUM ('anime', 'manga', 'game', 'tv', 'movie');
CREATE TYPE difficulties AS ENUM('easy', 'medium', 'hard');

CREATE TABLE charades (
  id SERIAL PRIMARY KEY,
  category categories NOT NULL,
  hint TEXT NOT NULL,
  puzzle TEXT NOT NULL,
  solution TEXT NOT NULL,
  difficulty difficulties NOT NULL,
  userID NUMERIC NOT NULL CHECK (userID >= 0 AND userID < 18446744073700000000),
  public BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE feeds (
	id SERIAL PRIMARY KEY,
	server_id BIGINT NOT NULL,
	channel_id BIGINT NOT NULL,
	manga_id BIGINT NOT NULL
);

CREATE TABLE roles (
	id SERIAL PRIMARY KEY,
	server_id BIGINT NOT NULL,
	roles TEXT NOT NULL
);