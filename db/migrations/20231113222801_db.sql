CREATE TABLE users (
    id uuid primary key default gen_random_uuid(),
    username text not null,
    email text unique not null,
    pswd_hash text not null,
    pswd_salt text not null,
    -- limit access rate
    last_access timestamp
);

INSERT INTO users (
        username,
        email,
        pswd_hash,
        pswd_salt,
        last_access
    )
VALUES (
        'admin',
        'admin@admin.com',
        -- password: pswd1234, hash(pswd1234pjZKk6A8YtC8$9p&UIp62bv4PLwD7@dF)
        '7c44575b741f02d49c3e988ba7aa95a8fb6d90c0ef63a97236fa54bfcfbd9d51',
        'pjZKk6A8YtC8$9p&UIp62bv4PLwD7@dF',
        now()
    );