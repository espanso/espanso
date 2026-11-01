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

## macOS event loop regression tests

Run `bash scripts/test_macos_shutdown.sh` on macOS to check worker shutdown,
the Reload config menu action, shutdown requested on the main thread, and the
launcher's headless event loop. The standalone AppKit tests run in separate
processes so that each case owns the Cocoa main thread. They check shutdown
threading, receiver lifetime, late callbacks, and event loop wake-up without
using `abortModal`.
