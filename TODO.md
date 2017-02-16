# To Do list


Before production
- Choose the posttest questions
- Edit more sentences
- Change the welcome screen
- Change the nag message

- Async rate limiting?
- Monitoring
- DoS prevention to HAProxy

- Config files to repo

After starting production
- Edit more sentences

Stretch goals:
- Refactor the error section and bug reporting functionality to be part of the base template
- Redirect to login page on 401 and when not logged in
- Support multiple audio file formats
- FIX: the annoying UI bug on SafarI where scale-animated buttons ignore clicks sometimes, if that's even possible
- FIX: prevent multiple sessions having the quiz open at the same time (possible ear-mark answers with session-id, think a bit about concurrency!)
- IMPROVEMENT: check if group names and event names are possible to enumify
