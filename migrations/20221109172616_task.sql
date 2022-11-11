-- Add migration script here
CREATE TABLE IF NOT EXISTS task (
    id  INTEGER PRIMARY KEY, -- sqlite automatically uses "integer primary key"  as synonym for ROWID column 
    task varchar(255) NOT NULL

);