# Contributing

Welcome to Espanso!
We are very happy to have you and we thank you for considering contributing!

We would like to order the contributions like this:

- Before you start hacking, it is important to **ask first** if your idea or bugfix is in order.
  Create an issue or come and say hi in the `#dev` channel for fixing bugs and adding features involved with coding logic, and `#documentation` for updating primarly the website. Join us at [the discord][`espanso` discord].
  We don't bite!.

  It would be sad that you do the effort to clone the project, successfully make the PR, but it wasn't in our plans or there is another PR that is currently addressing that issue.

- After the PR is submitted, the workflows will start to lint, check and test the changes. Please try to stay all green ✅.
- Most of the time we take some time to respond. Sorry, we are few and there SO much to do here!

## General guidelines and philosophy

This is a list of things we would like to have and mantain across time. Please do your best to abide by.

- We are geared towards a mostly-rust codebase, except on interactions with OS native modules (eg. C++ on Windows and Objective-C on macOS). We decided to stay on the native langauges on each platform, but if it's possible to make a change into rust, submit a PR and we'll see what can we do.
- Everything should be explained on the documentation via drawings, markdown files, etc, but it is important to make it clear. There will always be some new guy or gal into the project we want to welcome 😄.
- Use clear variable names and try to avoid confusing abbreviations. Think that your peers may not be fully fluent in english 💬.

[`espanso` discord]: https://discord.gg/4QARseMS6k

## Developing

We would like the rust code:

- to be formatted via `rustfmt`

  ```console
  cargo fmt --all
  ```

- to abide by the `clippy` ruleset we currently use

  ```console
  cargo clippy --workspace
  ```

- to approve the tests by running

  ```console
  cargo test --workspace
  ```

- to be compiled with `stable`, not `nightly`
- prefer not to use macros, if possible. Try to use functions or generics.

And C / C++ code:

- we would like to use `clang-format`
- and we would like to use `clang-tidy`

but the code submitted is yet unformatted and untidy. Work in progress!

### Tests

It is good practice to cover your changes with a test. Also, try to think about corner cases and various ways how your changes could break. Cover those in the tests as well.

Tests can be found in different places:

- `/tests`
- `src/tests`
- command examples
- crate-specific tests

## `git`

`git` is a powerful but a bit complex tool to use, and there are many criteria around the internet. We normally:

- squash the commits when we merge a PR
- TODO: setup git hooks with [`rusty-hook`](https://github.com/swellaby/rusty-hook)

---

### A good PR makes a change!

As a result of this PR-centric strategy and the general goal that the reviewers should easily understand your change, the **PR title and description matters** a great deal!

> **Note**
> Try to follow the suggestions in our PR message template to make sure we can quickly focus on the technical merits and impact on the users.

#### A PR should limit itself to a single functional change or related set of same changes

Mixing different changes in the same PR will make the review process much harder. A PR might get stuck on one aspect while we would actually like to land another change. Furthermore, if we are forced to revert a change, mixing and matching different aspects makes fixing bugs or regressions much harder.

Thus, please try to **separate out unrelated changes**!
**Don't** mix unrelated refactors with a potentially contested change.
Stylistic fixes and housekeeping can be bundled up into singular PRs.
