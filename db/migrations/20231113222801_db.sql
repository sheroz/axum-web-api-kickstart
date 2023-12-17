-- create users table
CREATE TABLE users
(
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username    TEXT        NOT NULL,
    email       TEXT UNIQUE NOT NULL,
    pswd_hash   TEXT        NOT NULL,
    pswd_salt   TEXT        NOT NULL,
    created_at  TIMESTAMP   NOT NULL,
    updated_at  TIMESTAMP   NOT NULL
);

-- populate users table
INSERT INTO users (username,
                   email,
                   pswd_hash,
                   pswd_salt,
                   created_at,
                   updated_at)
VALUES ('admin',
        'admin@admin.com',
           -- password: pswd1234, hash(pswd1234pjZKk6A8YtC8$9p&UIp62bv4PLwD7@dF)
        '7c44575b741f02d49c3e988ba7aa95a8fb6d90c0ef63a97236fa54bfcfbd9d51',
        'pjZKk6A8YtC8$9p&UIp62bv4PLwD7@dF',
        now(),
        now());