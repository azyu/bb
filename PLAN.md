# PLAN

## Objective
- Build a Bitbucket CLI with a `gh`-like structure focused on Bitbucket Cloud MVP.

## Phases
1. Research and command/API scope definition.
2. Project bootstrap (toolchain and repository structure).
3. Core implementation (`auth`, `api`, `repo`, `pr`).
4. Validation and release readiness.

## Success Criteria
- Core commands are documented and implemented for Cloud MVP.
- Authentication, pagination, and output modes work as specified.
- Basic verification workflow is documented and repeatable.

## Current Phase
- Phase: 4 (Validation and release readiness)
- Owner: agent
- Notes: Core wrappers for `auth`, `api`, `repo list`, `pr list/create`, `pipeline list/run`, `wiki list/get/put`, `issue list/create/update`, and `completion` are implemented with tests. Auth supports login/status/logout plus both Basic (API token with username/email) and Bearer modes. Next focus is release hardening and stronger wiki git auth secret handling.
