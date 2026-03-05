---
agent: agent
---

### Description

At the moment the code is fetching all the Github organizations and teams that the user has access to. This is not ideal as it can be a very large list and it is not necessary to fetch all of them. Instead, we should only fetch the teams that are relevant to the user.

The Github API allows to fetch the teams that a user is a member of. This can be done by using the following endpoint: `GET /user/teams`. This will return a list of teams that the user is a member of, along with the organization that each team belongs to.

Check that Octocrab allows for such a request and if it does, update the code to fetch only the teams that the user is a member of. If Octocrab does not support this endpoint, we may need to implement a custom request to the Github API to achieve this.

### Implementation steps
[] Check that the Octocrab library supports fetching the teams that a user is a member of using the `GET /user/teams` endpoint.
[] If Octocrab supports this endpoint, update the code to fetch only the teams that the user is a member of instead of fetching all teams.
[] If Octocrab does not support this endpoint, implement a custom request to the Github API to fetch the teams that the user is a member of and update the code accordingly.
[] Create and update tests to ensure that the new functionality works as expected and that only the relevant teams are fetched for the user.