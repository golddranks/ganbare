# To Do list

Today
- Ensure the words are introduced in pairs; skill-based new_word
- Introducing new words is rate-limited
- Check the group when giving new quizes: tester_input, tester_output, tester_nosurvey
- Refine the timing algorithms


- Introduce higher-level questions
- Impl the test tool (asks questions from both groups)


Plan:
- Do a group view, add anonymous groups and anonymous aliases
- Assign the group test_subjecs to the subects
- Impl a tool that then splits the group into half and assigns them to different groups




Stretch goals:
- FIX: unpublishing doesn't have effect on things showing up on due list. This needs filtering the source query BEFORE left_outer_join (in get_due_items()), but that's impossible with the current Diesel.
- Test with different browsers
- FIX: contenteditable adds <br> at the end
- FEATURE: download images server-side and replace the outbound links with links to the own static folder
- FEATURE: add answers in the manager screen
- IMPROVEMENT: check if group names and event names are possible to enumify
