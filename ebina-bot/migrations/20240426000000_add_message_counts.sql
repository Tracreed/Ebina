CREATE TABLE message_counts (
    server_id BIGINT NOT NULL,
    date DATE NOT NULL,
    count INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (server_id, date)
);
