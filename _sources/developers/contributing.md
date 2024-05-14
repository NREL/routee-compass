
# Contributing to RouteE Compass 

This simple guide will walk you through the process of contributing to RouteE Compass.

## Identifying an Issue

1. Go to the “Issues” tab in the repository.
1. Look for issues with labels such as “good first issue” if you’re new to the project.
1. Read the issue description and any associated comments to make sure you understand the problem and its requirements.
1. Choose an issue that aligns with your skills and interests.

## Letting Others Know You’re Working on the Issue

1. Leave a comment on the issue, stating that you’d like to work on it. This helps prevent duplication of effort and lets us know someone is addressing the problem.
1. Wait for a response from the maintainers or the issue creator. They may provide additional information or guidance, or assign the issue to you.

Forking the Repository

1. Navigate to the main RouteE Compass page on GitHub.
1. Click on the “Fork” button in the upper right corner. This creates a copy of the repository under your GitHub account.
1. Clone your forked repository to your local machine by clicking the “Code” button and copying the URL. Then, open your terminal or command prompt and run git clone [URL], replacing [URL] with the copied URL.

## Installing the development environment

1. Create and activate a new virtual environment with the tool of your choice (e.g. venv, conda, pipenv, etc.).
1. Install the package and dependencies by running pip install -e ".[dev]". This will install the package in editable mode, so you can make changes to the code and see them reflected in your environment.

## Opening a Pull Request

1. Create a new branch in your local repository by running git checkout -b [branch-name], replacing [branch-name] with a descriptive name for your changes.
1. Make changes to the code, addressing the issue you’ve chosen to work on.
1. Commit your changes with a meaningful commit message. Use git add [file] to stage changes, and git commit -m “[commit-message]” to commit them.
1. Push your changes to your forked repository using git push origin [branch-name].
1. Go to your forked repository on GitHub and click on the “Pull requests” tab.
1. Click on the “New Pull Request” button, and select your branch in the “compare” dropdown menu.
1. Review your changes and click “Create Pull Request”. Add a descriptive title and a detailed explanation of the changes you made.
1. Submit your pull request. The project maintainers will review your changes and provide feedback or merge your changes into the main repository.

Thanks for taking the time to contribute to RouteE Compass!
