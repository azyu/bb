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
- Phase: 3 (Core implementation)
- Owner: agent
- Notes: Go CLI skeleton, command contracts, shared API client (auth + pagination), and SemVer+hash version command are implemented for Cloud MVP baseline.
