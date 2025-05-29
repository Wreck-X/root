CREATE TABLE StatusUpdateHistory (
	update_id SERIAL PRIMARY KEY,
	member_id INT REFERENCES Member(member_id) ON DELETE CASCADE,
	date DATE NOT NULL,
	is_updated BOOLEAN NOT NULL DEFAULT FALSE,
	UNIQUE (member_id, date)
);
