-- Your SQL goes here
ALTER TABLE user_metrics ALTER max_quizes_since_break SET DEFAULT 12;
ALTER TABLE user_metrics ALTER max_quizes_today SET DEFAULT 36;
UPDATE user_metrics SET max_quizes_since_break = 12 WHERE max_quizes_since_break = 20;
UPDATE user_metrics SET max_quizes_today = 36 WHERE max_quizes_today = 80;