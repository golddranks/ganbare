# To Do list

Today
- Design an alternative mode for quizes: traditional/output
- Do a group view, add anonymous groups and anonymous aliases
- Refine the timing algorithms, especially w.r.t breaks and higher-level questions


Plan:
- Assign the group test_subjecs to the subects
- Impl a tool that then splits the group into half and assigns them to different groups
- Check the group when giving new quizes
- Impl the test tool (asks questions from both groups)
- Desgin a survey


Stretch goals:
- Improve the HTTP API: / redirects to subpages, accepting things will be a POST /ok call following by redirect, each subpage has it's own URL, / is reserved for the main quiz. /login too. 
- Modularize the project: (server.rs: api module, manage module?)
- Do something for the copyright footer when using with small-screened devices
- Test with different browsers
- FIX: multiple overlapping sessions getting stale
- FIX: contenteditable adds <br> at the end
- FEATURE: download images server-side and replace the outbound links with links to the own static folder
- FEATURE: add answers in the manager screen
- IMPROVEMENT: check if group names and event names are possible to enumify
