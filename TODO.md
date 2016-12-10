# To Do list

Today

- Ad-blockes block outbound links
- Make manager + audio texts searchable
- Make the texts in order
- Implement forgot password feature
- Implement nag emails
- Implement user metrics viewing GUI
- Fix the answer button UI bug on IE
- Fix the survey UI bug on mobile

- Publish more words
Tomorrow
- Introduce higher-level questions
- Impl the test tool (asks questions from both groups)


Weekend:
- Implement anonymous groups and anonymous aliases
- Impl a tool that then splits the group into half and assigns them to different groups




Stretch goals:
- Redirect to login page on 401 and when not logged in
- Support multiple audio file formats
- FIX: the annoying UI bug on SafarI where scale-animated buttons ignore clicks sometimes, if that's even possible
- FIX: prevent multiple sessions having the quiz open at the same time (possible ear-mark answers with session-id, think a bit about concurrency!)
- FIX: unpublishing doesn't have effect on things showing up on due list. This needs filtering the source query BEFORE left_outer_join (in get_due_items()), but that's impossible with the current Diesel.
- Test with different browsers
- FIX: contenteditable adds <br> at the end
- FEATURE: add answers in the manager screen
- IMPROVEMENT: check if group names and event names are possible to enumify
