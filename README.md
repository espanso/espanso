# ![Espanso Logo](images/espanso-dark.png#gh-dark-mode-only) ![Espanso Logo](images/espanso-light.png#gh-light-mode-only)

> *A cross-platform Text Expander written in Rust*

![GitHub release (latest by date)](https://img.shields.io/github/v/release/espanso/espanso)
![Maintenance](https://img.shields.io/badge/Maintained-yes-green.svg)
![Language](https://img.shields.io/badge/language-rust-orange)
![License](https://img.shields.io/github/license/espanso/espanso)

![Platforms](https://img.shields.io/badge/platforms-Windows%2C%20macOS%20and%20Linux-blue)

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/espanso/espanso)

|         Example: 2019          |              Example: 2025              |
| :----------------------------: | :-------------------------------------: |
| ![example](images/example.gif) | ![example2025](images/example-2025.gif) |

## Quick Links

* [espanso website](https://espanso.org)
* [espanso hub](https://hub.espanso.org/)

### What is a Text Expander?

A *text expander* is a program that detects when you type
a specific **keyword** and replaces it with **something else**.
This is useful in many ways:

* **Save a lot of typing**, expanding common sentences
* Create **system-wide** code snippets
* Execute **custom scripts**
* Use **emojis** like a pro
* System-wide 'autocorrect' specific to you

## Key Features

* Cross-platform (**Windows**, **macOS**, **Linux**)
* Privacy-first (100% local, no tracking)
* Works with almost **any program**
* **Emoji** support 😄
* **Image** support
* Includes a powerful **Search Bar** 🔎
* **Date** expansion support
* **Custom scripts** support
* **Shell commands** support
* **App-specific** configurations
* Support [Forms](https://espanso.org/docs/matches/forms/)
* Expandable with **packages**
* Built-in **package manager** for [espanso hub](https://hub.espanso.org/)
* File based configuration
* Support Regex triggers
* Experimental Wayland support
* Written in Rust (Fast, Reliable)

## Community & Support
* 💬 [espanso Discord Server](https://discord.gg/DFcCNDg7bB)
* 📖 [official documentation](https://espanso.org/docs/)
* 💬 [official Subreddit](https://www.reddit.com/r/espanso/)
* 🐛 [Report Issues](https://github.com/espanso/espanso/issues)
* 💡 [Feature Requests](https://github.com/espanso/espanso/discussions)

## Quick Start Examples

You can create additional files to organize your matches any way you want.<br />
Make sure to adhere to proper YAML spacing.
```yaml
matches:
  - trigger: ":hello"
    replace: "Hi There!"
  - triggers: [":test1", ":test2"]
    replace: "These both expand to the same thing"
```
## Team Members and Contributors

### Team

[Federico Terzi](https://github.com/federico-terzi) (Creator of espanso)<br />
Rest of team in Alphabetical Order<br />
[Archigos](https://github.com/Archigos) (Lead Maintainer)<br />
[Auca](https://github.com/AucaCoyan) (Previous Lead Maintainer)<br />
[n8henrie](https://github.com/n8henrie)<br />
[smeech](https://github.com/smeech)<br />

You can also see the up to date list of Team Members [here](https://github.com/orgs/espanso/people)

### Contributors

So many people have helped the project along the way. Thank you all!

[![Image](https://contrib.rocks/image?repo=espanso/espanso)](https://github.com/espanso/espanso/graphs/contributors)

## Sponsors

We want to thank SignPath.io for code signing the Windows binaries ❤️

## Donations

espanso is a free, open-source software project created by [Federico Terzi](https://github.com/federico-terzi) and now maintained by a small team.<br />
If you liked the project and would like to support further development,
please consider  making a small donation, it really helps :)

### Current Options

| PayPal | Coming Soon |
| :----: | :---------: |
| [![Donate with PayPal](images/donate.gif)](https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=FHNLR5DRS267E&source=url) | |

## Remarks

* Thanks to [libxdo](https://github.com/jordansissel/xdotool) and [xclip](https://github.com/astrand/xclip), used to implement the Linux port.
* Thanks to [libxkbcommon](https://xkbcommon.org/) and [wl-clipboard](https://github.com/bugaevc/wl-clipboard), used to implement the Wayland port.
* Thanks to [wxWidgets](https://www.wxwidgets.org/) for providing a powerful cross-platform GUI library.
* Free code signing provided by SignPath.io, certificate by SignPath Foundation.

## License

espanso was created by [Federico Terzi](http://federicoterzi.com)
and is licensed under the [GPL-3.0 license](/LICENSE).
