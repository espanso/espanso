![espanso](images/logo_extended.png)

> A cross-platform Text Expander written in Rust

![GitHub release (latest by date)](https://img.shields.io/github/v/release/federico-terzi/espanso)
![Language](https://img.shields.io/badge/language-rust-orange)
![Platforms](https://img.shields.io/badge/platforms-Windows%2C%20macOS%20and%20Linux-blue)
![License](https://img.shields.io/github/license/federico-terzi/espanso)

![example](images/example.gif)

Visit the [espanso website](https://espanso.org).

#### What is a Text Expander?

A *text expander* is a program that detects when you type
a specific **keyword** and replaces it with **something else**. 
This is useful in many ways:
* **Save a lot of typing**, expanding common sentences.
* Create **system-wide** code snippets.
* Execute **custom scripts**
* Use **emojis** like a pro.

___

## Key Features

* Works on **Windows**, **macOS** and **Linux**
* Works with almost **any program**
* Works with **Emojis** 😄
* Works with **Images**
* Includes a powerful **Search Bar** 🔎
* **Date** expansion support
* **Custom scripts** support
* **Shell commands** support
* **App-specific** configurations
* Support [Forms](https://espanso.org/docs/forms/)
* Expandable with **packages**
* Built-in **package manager** for [espanso hub](https://hub.espanso.org/)
* File based configuration
* Support Regex triggers
* Experimental Wayland support

## Get Started

Visit the [official documentation](https://espanso.org/docs/).

## Wayland support

Please look at the documentation for details. Application specific filters
(filter_title, filter_class, filter_exec) currently only work und Wayland with Gnome
version 41. In addition, a Gnome shell extension
(https://extensions.gnome.org/extension/4974/window-calls-extended/)
is required! Without this extension, espanso has no access to active window information.


## Support

If you need some help to setup espanso, want to ask a question or simply get involved
in the community, [Join the official Subreddit](https://www.reddit.com/r/espanso/)! :)



## Donations

espanso is a free, open source software developed in my (little) spare time.
If you liked the project and would like to support further development, 
please consider making a small donation, it really helps :)

[![Donate with PayPal](images/donate.gif)](https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=FHNLR5DRS267E&source=url)

## Contributors

Many people helped the project along the way, thank you to all of you!

<a href="https://github.com/federico-terzi/espanso/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=federico-terzi/espanso" />
</a>

## Remarks

* Thanks to [libxdo](https://github.com/jordansissel/xdotool) and [xclip](https://github.com/astrand/xclip), used to implement the Linux port.
* Thanks to [libxkbcommon](https://xkbcommon.org/) and [wl-clipboard](https://github.com/bugaevc/wl-clipboard), used to implement the Wayland port.
* Thanks to [wxWidgets](https://www.wxwidgets.org/) for providing a powerful cross-platform GUI library.

## License

espanso was created by [Federico Terzi](http://federicoterzi.com)
and is licensed under the [GPL-3.0 license](/LICENSE).
