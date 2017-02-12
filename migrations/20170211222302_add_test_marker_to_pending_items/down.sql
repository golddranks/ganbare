ALTER TABLE pending_items DROP COLUMN test_item;
ALTER TABLE user_metrics DROP COLUMN streak_skill_bump_criteria;
UPDATE user_metrics SET initial_delay=10000;
