ALTER TABLE pending_items ADD COLUMN test_item BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE user_metrics ADD COLUMN streak_skill_bump_criteria INTEGER NOT NULL DEFAULT 4;
UPDATE user_metrics SET initial_delay=28800;
