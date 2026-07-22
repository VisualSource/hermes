CREATE TABLE IF NOT EXISTS servers (
    id BLOB NOT NULL PRIMARY KEY,
    name TEXT NOT NULL
    owner_id BLOB NOT NULL,
    created_at DATETIME NOT NULL,

    FOREIGN KEY(owner_id)
        REFERENCES users(id)
            ON DELETE RESTRICT
            on UPDATE CASCADE
) strict;

CREATE TABLE IF NOT EXISTS server_members (
    id TEXT NOT NULL PRIMARY KEY,
    server_id BLOB NOT NULL,
    user_id BLOB NOT NULL,

    FOREIGN KEY(server_id) 
        REFERENCES servers(id)
            ON DELETE CASCADE
            ON UPDATE CASCADE

    FOREIGN KEY(user_id)
        REFERENCES users(id)
            ON DELETE CASCADE
            on UPDATE CASCADE
) strict;
CREATE INDEX IF NOT EXISTS idx_server_members_server ON server_members(server_id, user_id);

CREATE TABLE IF NOT EXISTS roles (
    id BLOB NOT NULL PRIMARY KEY,
    server_id BLOB NOT NULL,
    name TEXT NOT NULL,
    mask BIGINT NOT NULL,
    FOREIGN KEY(server_id) 
        REFERENCES servers(id) 
            ON DELETE CASCADE 
            ON UPDATE CASCADE
) strict;

CREATE TABLE IF NOT EXISTS role_members (
    role_id BLOB NOT NULL,
    member_id BLOB NOT NULL,
    
    PRIMARY KEY(role_id, member_id),
    FOREIGN KEY(role_id)   
        REFERENCES roles(id)          
            ON DELETE CASCADE 
            ON UPDATE CASCADE,
    FOREIGN KEY(member_id) 
        REFERENCES server_members(id) 
            ON DELETE CASCADE 
            ON UPDATE CASCADE
) strict;
CREATE INDEX IF NOT EXISTS idx_role_members_member ON role_members(member_id);

CREATE TABLE IF NOT EXISTS channels (
    id BLOB NOT NULL PRIMARY KEY,
    kind TEXT NULL NULL CHECK(kind IN ('text','voice','dm')),
    server_id BLOB -- null when being used for dm kind
    name TEXT NOT NULL,
    roles BIGINT NOT NULL,
    category TEXT

    FOREIGN KEY(server_id) 
        REFERENCES servers(id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
) strict;
CREATE INDEX IF NOT EXISTS idx_channels_server ON channels(server_id);


CREATE TABLE IF NOT EXISTS dm_participants (
    id TEXT NOT NULL PRIMARY KEY,
    channel_id BLOB NOT NULL,
    user_a_id BLOB NOT NULL,
    user_b_id BLOB NOT NULL

    FOREIGN KEY(channel_id) 
        REFERENCES channels(id)
            ON DELETE CASCADE
            ON UPDATE CASCADE

    FOREIGN KEY(user_a_id)
        REFERENCES users(id)
            ON DELETE RESTRICT
            on UPDATE CASCADE
    FOREIGN KEY(user_b_id)
        REFERENCES users(id)
            ON DELETE RESTRICT
            on UPDATE CASCADE
) strict;

CREATE TABLE IF NOT EXISTS messages (
    id BLOB NOT NULL PRIMARY KEY,
    channel_id BLOB NOT NULL,
    user_id BLOB NOT NULL,
    content TEXT,

    created_at DATETIME NOT NULL,
    edited_at DATETIME,
    deleted_at DATETIME,
  
    FOREIGN KEY(user_id)
        REFERENCES users(id)
            ON DELETE CASCADE
            on UPDATE CASCADE

    FOREIGN KEY(channel_id) 
        REFERENCES channels(id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE 
) strict;
CREATE INDEX IF NOT EXISTS idx_messages_channel_ts ON messages(channel_id, created_at);

CREATE TABLE IF NOT EXISTS invites (
    id BLOB NOT NULL PRIMARY KEY,
    server_id BLOB NOT NULL,
    created_by BLOB,
    expires_at DATETIME,
    max_uses INT NOT NULL DEFAULT 2,
    uses INT NOT NULL DEFAULT 0
    revoked BOOLEAN NOT NULL DEFAULT FALSE,

    FOREIGN KEY(server_id) 
        REFERENCES servers(id)
            ON DELETE CASCADE
            ON UPDATE CASCADE

    FOREIGN KEY(created_by) 
        REFERENCES users(id)   
            ON DELETE SET NULL 
            ON UPDATE CASCADE
) strict;