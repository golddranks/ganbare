# To Do list

Need to do before release:

- Make a retelling task event for pretest and posttest
- Finalize and test posttest (mainly copy&paste from pretest)
- Edit some (5) sentences
- Implement question priority to ensure the learners know a common sub-set of words
- Check that asking sentences work


Stretch goals:
- Remove old mail confirmations
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
