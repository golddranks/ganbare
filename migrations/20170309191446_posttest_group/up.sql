INSERT INTO user_groups (group_name) VALUES ('posttest');
UPDATE events SET required_group=(SELECT id FROM user_groups WHERE group_name='posttest') WHERE name='posttest_info' OR name='posttest' OR name='posttest_retelling' OR name='posttest_done';
UPDATE events SET published=true WHERE name='agreement' OR name='info' OR name='pretest_info' OR name='pretest_retelling' OR name='pretest_done' OR name='sorting_ceremony' OR name='posttest_info' OR name='posttest' OR name='posttest_retelling' OR name='posttest_done';
