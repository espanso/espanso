# Creating a PR

## Testing

It is good practice to cover your changes with a test. Also, try to think about corner cases and various ways how your changes could break. Cover those in the tests as well.

Tests can be found in 2 places:

- `/test/` folder (we don't have any cases today, but it's an alternative)
- within the same file of the function, in a `tests` mod

## Creating a PR

Firt and foremost: **A good PR makes a change!**. Let's chat some basic theory before going to the practice

Because of this PR-centric strategy and the goal that the reviewers should easily understand your change, the **PR title and description matters** a great deal!

> **Note**
> Try to follow the suggestions in our PR message template to make sure we can quickly focus on the technical merits and impact on the users.

### A PR should limit itself to a single functional change or related set of same changes

Mixing different changes in the same PR will make the review process much harder. A PR might get stuck on one aspect while we would actually like to land another change. Furthermore, if we are forced to revert a change, mixing and matching different aspects makes fixing bugs or regressions much harder.

Thus, please try to **separate out unrelated changes**!
**Don't** mix unrelated refactors with a potentially contested change.
Stylistic fixes and housekeeping can be bundled up into singular PRs.

### Going to the practice

We would like the Rust code:

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

- we would like to use a formatter, like `clang-format` (we can use [run-clang-format](https://github.com/lmapii/run-clang-format)
to format folders recursively)
- and we would like to use `clang-tidy`, but we need a Compilation database in order to use `clang-tidy`
 (also, we can use [run-clang-tidy](https://github.com/lmapii/run-clang-tidy) to help too!)

Today our submitted code is yet untidy. Work in progress!

### `git`

`git` is a powerful but a bit complex tool to use, and there are many criteria around the internet. We normally:

- squash the commits when we merge a PR
- TODO: setup git hooks with some tool

