-- Your SQL goes here
CREATE TABLE emails (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    sender TEXT NOT NULL,
    subject TEXT,
    body TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE recipients (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    email_id INTEGER NOT NULL,
    recipient TEXT NOT NULL,
    FOREIGN KEY(email_id) REFERENCES emails(id) ON DELETE CASCADE,
    UNIQUE(email_id, recipient)
);

CREATE INDEX idx_recipient ON recipients(recipient);
