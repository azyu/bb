# Bitbucket Cloud Endpoint Map (MVP)

- Workspaces: `GET /user/workspaces`
- Repositories list: `GET /repositories/{workspace}`
- Pull requests list: `GET /repositories/{workspace}/{repo_slug}/pullrequests`
- Pull request create: `POST /repositories/{workspace}/{repo_slug}/pullrequests`
- Pull request get: `GET /repositories/{workspace}/{repo_slug}/pullrequests/{id}`
- Pull request update: `PUT /repositories/{workspace}/{repo_slug}/pullrequests/{id}`
- Pull request merge: `POST /repositories/{workspace}/{repo_slug}/pullrequests/{id}/merge`
- Pull request approve: `POST /repositories/{workspace}/{repo_slug}/pullrequests/{id}/approve`
- Pull request unapprove: `DELETE /repositories/{workspace}/{repo_slug}/pullrequests/{id}/approve`
- Pull request request changes: `POST /repositories/{workspace}/{repo_slug}/pullrequests/{id}/request-changes`
- Pull request remove request changes: `DELETE /repositories/{workspace}/{repo_slug}/pullrequests/{id}/request-changes`
- Pull request decline: `POST /repositories/{workspace}/{repo_slug}/pullrequests/{id}/decline`
- Pull request comment create: `POST /repositories/{workspace}/{repo_slug}/pullrequests/{id}/comments`
- Pull request comment update: `PUT /repositories/{workspace}/{repo_slug}/pullrequests/{id}/comments/{comment_id}`
- Pull request comments list: `GET /repositories/{workspace}/{repo_slug}/pullrequests/{id}/comments`
- Pull request diff: `GET /repositories/{workspace}/{repo_slug}/pullrequests/{id}/diff`
- Pull request statuses: `GET /repositories/{workspace}/{repo_slug}/pullrequests/{id}/statuses`
- Pull request activity: `GET /repositories/{workspace}/{repo_slug}/pullrequests/{id}/activity`
- Pipelines list: `GET /repositories/{workspace}/{repo_slug}/pipelines`
- Pipeline get: `GET /repositories/{workspace}/{repo_slug}/pipelines/{uuid}`
- Pipeline steps: `GET /repositories/{workspace}/{repo_slug}/pipelines/{uuid}/steps`
- Pipeline log: `GET /repositories/{workspace}/{repo_slug}/pipelines/{uuid}/steps/{step_uuid}/log`
- Pipeline run: `POST /repositories/{workspace}/{repo_slug}/pipelines`
- Issues list: `GET /repositories/{workspace}/{repo_slug}/issues`
- Issue create: `POST /repositories/{workspace}/{repo_slug}/issues`
- Issue update: `PUT /repositories/{workspace}/{repo_slug}/issues/{id}`

Notes:
- Observed behavior (not an officially documented contract): `Pipeline get` also accepts a build number in place of `{uuid}`, and `Pipelines list` ignores `q=build_number=<n>` (it returns the whole collection). Resolve build selectors with `GET /repositories/{workspace}/{repo_slug}/pipelines/{build_number}`; a missing build returns 404 with detail `Pipeline with build number '<n>' not found in repository ...`, which is what confirms the segment is read as a build number.
- Wiki commands are intentionally excluded from this map because this project handles wiki operations through the wiki Git repository, not Bitbucket Cloud REST endpoints.

Use `bb api` for coverage gaps before adding a dedicated wrapper command.
