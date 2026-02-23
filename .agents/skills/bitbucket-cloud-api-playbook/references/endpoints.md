# Bitbucket Cloud Endpoint Map (MVP)

- Workspaces: `GET /user/workspaces`
- Repositories list: `GET /repositories/{workspace}`
- Pull requests list: `GET /repositories/{workspace}/{repo_slug}/pullrequests`
- Pull request create: `POST /repositories/{workspace}/{repo_slug}/pullrequests`
- Pipelines list: `GET /repositories/{workspace}/{repo_slug}/pipelines`
- Pipeline run: `POST /repositories/{workspace}/{repo_slug}/pipelines`
- Issues list: `GET /repositories/{workspace}/{repo_slug}/issues`

Use `bb api` for coverage gaps before adding a dedicated wrapper command.
