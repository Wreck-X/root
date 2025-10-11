-- Add migration script here
ALTER TABLE Member
ADD COLUMN github_user varchar(255);