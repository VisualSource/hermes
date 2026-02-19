CREATE TABLE IF NOT EXISTS users (
    id BLOB NOT NULL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    psd_hash TEXT NOT NULL,
    avatar TEXT,
    created_at DATETIME NOT NULL,
    mfa BOOLEAN NOT NULL DEFAULT FALSE,
    email TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS keys (
    id BLOB NOT NULL PRIMARY KEY,
    user_id BLOB NOT NULL,
    public_key TEXT NOT NULL,
    FOREIGN KEY(user_id) 
        REFERENCES users(id) 
            ON DELETE CASCADE 
            ON UPDATE NO ACTION
);