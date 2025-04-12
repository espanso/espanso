# Platform Tiers

We have different tiers to indicate different levels of commitment for each
architecture + Operative System.

In the Tier 1, we want to garantee the installation, configuration and general
usage of the app. This are the platforms we spend the majority of time.

- ✅ CI will always build and pass all the tests
- ✅ Releases are built for these platforms

Tier 2 is what can we do to our best efforts. This means sometimes we need to
break compatibility, or we need to stop supporting some features because there
are fewer users compared to the tier 1.

Tier 3 is unsupported. This is because we don't have hardware to compile or
repoduce the bugs. The shortage of people certainly doesn't help here. Help
needed!

### Tier 1: core

- x86_64 Windows 11 (latest)
- ARM64 (Silicon) macOS Sonoma (latest)
- x86_64 Ubuntu 24.04 (LTS)
- x86_64 Debian 12 (stable)

### Tier 2: our best effort

- x86_64 Windows 10 (close to end of service)
- x86_64 macOS (Intel)
- x86_64 Debian testing and Debian Sid (unstable)
- x86_64 Fedora distros
- x86_64 Arch linux

### Tier 3: unsupported

- Windows (32 bits, ARM)
- ChromeOS ([tracking issue](https://github.com/espanso/espanso/issues/2243))
- FreeBSD and the BSD family (tracking issue)
