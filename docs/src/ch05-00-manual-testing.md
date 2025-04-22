# Manual testing

It hurts a developer to say that an app needs manual testing, but in this case is
shorter to test the app manually instead of building lines upon lines just to
save a couple of minutes of inconvenience every 4 months.

As far as we can go with automated tests, we will, but for some cases like:

- Is it installable in current Debian stable? No problems with glibc?
- Does it work in a known-problematic app like a browser?
- Does anything seems off?

It's better a human to detect them, than CI. The last scenario we want to happen
is to reach users 😢.

## Checklist we want to do for manual testing

Take this as a shopping-list, not exhaustive:

- [ ] `espanso` in installable
- [ ] `espanso start` effectively starts the GUI
- [ ] `:espanso` makes an expansion

In linux:
- [ ] `espanso service start` doesn't crash
