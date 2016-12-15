# To Do list

Today:

- Publish more words


Tomorrow:
- Impl the test tool (asks questions from both groups)
- Implement anonymous groups and anonymous aliases
- Impl a tool that then splits the group into half and assigns them to different groups
- Introduce higher-level questions




Stretch goals:
- Event status on user management page
- Make "aika loppui" message to exercises too
- Make the survey to close the buttons when answered but before the answer is saved (needs to join the event of closing the buttons & getting response from server)
- Refactor the error section and bug reporting functionality to be part of the base template
- Redirect to login page on 401 and when not logged in
- Support multiple audio file formats
- FIX: the annoying UI bug on SafarI where scale-animated buttons ignore clicks sometimes, if that's even possible
- FIX: prevent multiple sessions having the quiz open at the same time (possible ear-mark answers with session-id, think a bit about concurrency!)
- FIX: unpublishing doesn't have effect on things showing up on due list. This needs filtering the source query BEFORE left_outer_join (in get_due_items()), but that's impossible with the current Diesel.
- Test with different browsers
- FIX: contenteditable adds <br> at the end
- FEATURE: add answers in the manager screen
- IMPROVEMENT: check if group names and event names are possible to enumify
