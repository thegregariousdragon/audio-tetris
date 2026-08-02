<RULE[project_branching_model]>
# Branching Model and Protected Main Branch

The `main` branch of this repository is protected and strictly requires pull request reviews. You CANNOT push directly to `main`.

When you are asked to make changes to the codebase, you MUST follow this Git workflow:
1. **Create a Branch**: Create and checkout a new branch with a descriptive name (e.g., `git checkout -b feature/dynamic-music`).
2. **Commit Changes**: Make your changes and commit them to this branch.
3. **Push Branch**: Push the branch to the remote repository (e.g., `git push -u origin feature/dynamic-music`).
4. **Create Pull Request**: Use the GitHub CLI to create a Pull Request against the `main` branch (`gh pr create --title "..." --body "..."`).
5. **Auto-Approve and Merge**: Once the Pull Request is created, use the GitHub CLI to approve it yourself (`gh pr review --approve`) and then merge it (`gh pr merge --squash --delete-branch`). This respects the branch protection while allowing you to complete the workflow automatically.
</RULE[project_branching_model]>
