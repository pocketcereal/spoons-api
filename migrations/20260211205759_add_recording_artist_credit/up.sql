-- Add artist_credit JSONB column to recordings table
ALTER TABLE recordings ADD COLUMN artist_credit JSONB NOT NULL DEFAULT '[]';
