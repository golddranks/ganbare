# To Do list

Today

- RELEASE 0.2

- More words to DB

- Introduce higher-level questions
- Impl the test tool (asks questions from both groups)


Plan:
- Do a group view, add anonymous groups and anonymous aliases
- Assign the group test_subjecs to the subects
- Impl a tool that then splits the group into half and assigns them to different groups




Stretch goals:
- FIX: prevent multiple sessions having the quiz open at the same time (possible ear-mark answers with session-id, think a bit about concurrency!)
- FIX: unpublishing doesn't have effect on things showing up on due list. This needs filtering the source query BEFORE left_outer_join (in get_due_items()), but that's impossible with the current Diesel.
- Test with different browsers
- FIX: contenteditable adds <br> at the end
- FEATURE: download images server-side and replace the outbound links with links to the own static folder
- FEATURE: add answers in the manager screen
- IMPROVEMENT: check if group names and event names are possible to enumify
