# Triage method

We want to be consistent in the order in which we bring value to our users.

For that we need to have a common ground and method to classify bugs from
enhancements, define what has high priority and what low priority, and so on.

It makes sense that order of priority follows the normal usage of the app, that
is the user:

1) Downloads the app, (assuming downloading the correct version for his/her
platform)
2) Install the app and there are no errors or warnings
3) Follows the welcome tutorial and Espanso triggers `Hello there!` correctly

From here there are a couple of approaches, probably:

- the user search how to add triggers and checks the configuration available
- the user lookup for some packages, finds the hub, and then installs some of
them.

In addition, some users can clone the repository from GitHub and compile from
source, if needed. So that's an additional area to cover from the beginning.

> ⚠️ IMPORTANT
>
> Any problem compiling, downloading, installing or following the welcome tutorial
will be categorized as `high priority`. If multiple problems appear, they are
sorted in the named order.

The statement above is valid under our [supported platforms](./ch02-01-platform-tiers.md),
 of course.

## So... what's medium or low priority then?

It's a difficult line to trace, but we have to draw it somewhere. Medium priority
is distinguised when it doesn't have `high` or `low` priority tags, so it's the
default for the issues.

`low priority` is acceptable for cases where:
- is an enhancement or feature that is difficult to implement, and we feel it
has very low impact on our users
- the above, especially when there is an alternative, non-native solution
- we don't have the water level up for the task (unsupported platforms, bugs
very difficult to reproduce, or short of staff)
- stuff we think is not important for the current state of the app

This doesn't mean the work is stopped on those areas, it's just stalled becase we
have loads of more important things to do. Note that many issues qualify to be
low-priority, but we want to solve them sooner than others.

Additionaly, we want to have a consistent value delivery, so we aim to have
around the same amount of high priority issues any time of the year.
