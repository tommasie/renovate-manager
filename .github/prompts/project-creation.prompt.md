---
agent: agent
---

You are an expert software developer.

Your task is to create a new project whose ultimate goal is to offer the user an user interface which allows to list, review, comment and approve all Renovate pull requests in all Github repositories of the Github Team associated to the user.

### Definitions
A Renovate pull request is a pull request created by the Renovate bot, which is a tool that automates the process of updating dependencies in software projects. Renovate pull requests typically contain updates to dependencies, such as new versions or security patches. A renovate pull request has the label `renovate`.

### Requisites
- The application is a CLI application written in Rust
- The application should use Cargo as the project management tool
- The application should use the Github CLI token for authentication and API calls
- The application should use octocrab as the Github API client crate

### The user interface
The user interface should be inspired by the [K9S UI](https://github.com/derailed/k9s), in that when the application is launched, it should display a list of all open Renovate pull requests of the repositories owned by the user's Github Team. The user should be able to navigate through the list using the keyboard and select a pull request to view more details, such as the pull request description, comments, and checks status. The user should also be able to approve or comment on the pull request directly from the application. The application should be closed by pressing the `q` key.

###  `list-prs` feature
In the first iteration of the application, it should list all open Renovate pull requests of the repositories owned by the user's Github Team.
The following information should be displayed for each pull request:
- Repository name
- Pull request title
- Pull request checks status

For this, write the source code as well as the unit tests.




