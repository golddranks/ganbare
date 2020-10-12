-- Your SQL goes here
ALTER TABLE user_metrics ALTER max_quizes_since_break SET DEFAULT 20;
ALTER TABLE user_metrics ALTER max_quizes_today SET DEFAULT 80;
UPDATE user_metrics SET max_quizes_since_break = 20 WHERE max_quizes_since_break = 12;
UPDATE user_metrics SET max_quizes_today = 80 WHERE max_quizes_today = 36;