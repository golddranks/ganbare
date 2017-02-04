# To Do list

Need to do before release:

- Check that asking sentences works
- Check that question priority works

- Remove/edit mispronounced items
- Choose the test questions

- Edit the high-priority items
- Final testing

Stretch goals:
- Make the survey to close the buttons when answered but before the answer is saved (needs to join the event of closing the buttons & getting response from server)
- Refactor the error section and bug reporting functionality to be part of the base template
- Redirect to login page on 401 and when not logged in
- Support multiple audio file formats
- FIX: the annoying UI bug on SafarI where scale-animated buttons ignore clicks sometimes, if that's even possible
- FIX: prevent multiple sessions having the quiz open at the same time (possible ear-mark answers with session-id, think a bit about concurrency!)
- FIX: unpublishing doesn't have effect on things showing up on due list. This needs filtering the source query BEFORE left_outer_join (in get_due_items()), but that's impossible with the current Diesel.
- FIX: contenteditable adds <br> at the end
- FEATURE: add answers in the manager screen
- IMPROVEMENT: check if group names and event names are possible to enumify
