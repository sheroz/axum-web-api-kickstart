CREATE TABLE IF NOT EXISTS users
(
    id              uuid primary key default gen_random_uuid(),
    username        text unique not null,
    password        text not null
);
