---
title: Passive Mode
layout: docs
---
> Note: at the moment, passive mode is still experimental and has to be enabled manually. Please see the "Enabling passive mode" section below.

In version 0.5.0, espanso introduced *Passive Mode*, a new feature which allows the user
to expand matches after typing them, instead of in realtime. The feature works as follows:

* Type a message containing any number of matches (passive mode matches are more limited, see the *Limitations* paragraph below)
* Select the text you want to process (conveniently done with the CTRL+A shortcut)
* Double press the `CTRL` key (you can customize this key).

As a result, espanso will copy the text, process it expanding all the matches, and then paste it back in the field.

![Passive Mode Example](/assets/images/passivemode1.gif)

### Enabling passive mode

Passive mode is still in its experimental stage, so it must be enabled manually. Add the following lines in the
`default.yml` file:

```yaml
enable_passive: true
passive_key: CTRL
```

Currently, the `passive_key` parameter accept the following alternatives: `CTRL`, `ALT`, `SHIFT` and `META` (Win key on Windows and Linux, CMD on macOS). If you'd like other possibilities, please open an issue.

### Format

Passive match triggers are a bit more limited than normal triggers. In particular, they have to start with a `:` prefix (though you can customize it, see below)
and should not contain spaces.

The default format of passive matches is:

```
:trigger/arg1/arg2/
```

But arguments are optional:

```
:trigger
```

> You can customize the default format by changing the configuration file, please take a look at the "Advanced Configuration" below.

### Arguments
One of the most requested features has always been *match arguments*. Due to the realtime nature
of espanso, this problem was very difficult to solve in a solid way. The solution is to use
passive mode, so that espanso can analyze whole sentences and execute a more complex elaboration.

![argument](/assets/images/passivemode2.gif)

Which can be obtained with the following:

```yaml
- trigger: ":greet"
  replace: "Hey $0$, how are you?\nIt's been a while!"
  passive_only: true
```

If you select `:greet/Jon/` and trigger the passive mode, the match will be expanded producing:

```
Hey Jon, how are you?
It's been a while!
```

The `$0$` keyword indicates where the argument should be placed, and you can also pass multiple arguments, so 
that they becomes `$1$`, `$2$`, ecc.

Notice the `passive_only` keyword, which makes espanso ignore the match when typing it (otherwise, espanso would
expand it right away).

The really powerful thing is that you can **pass these arguments to the shell or custom scripts** as well:

#### Integration with Shell

![argumentshell](/assets/images/passivemode3.gif)

This can be done by including `$0`, `$1` in the `cmd` parameter:

```yaml
- trigger: ":rev"
  replace: "{{output}}"
  passive_only: true
  vars:
    - name: output
      type: shell
      params:
        cmd: "echo $0 | rev"
        trim: true
```

**For Windows users**: instead of `$0`, you must use `%0`.

#### Integration with Scripts

Using the `inject_args` parameter, arguments will be appended to the given list when launching a program. For example:

```yaml
- trigger: ":pyscript"
  replace: "{{output}}"
  vars:
    - name: output
      type: script
      params:
        inject_args: true
        args:
          - python
          - /path/to/your/script.py
```

At this point, if you expand `:pyscript/hello/`, your script will receive "hello" as the first argument.

### Limitations

* **Passive mode does not work in terminals**. Unfortunately, because this feature heavily uses selections
and copy/pasting to work, I still haven't figured out a way to reliably make them work in terminals.

* **Matches have to start with a specific character**. The default character is `:`, but that can be customized
by changing the `passive_match_regex` parameter. This constraint has been added to improve the analysis efficiency.

* **Passive matches do not support images**.

### Advanced Customization

If you don't like the `:trigger/arg1/arg2/` syntax, you can customize it by changing a few parameters in your `default.yml` config as follow:

#### `passive_match_regex`

With the `passive_match_regex` you can customize the main trait of the passive matches, such as the prefix character and the **external** argument separators.
By default, it has the following value (notice the `\\` escaping which is mandatory):

```yaml
passive_match_regex: "(?P<name>:\\p{L}+)(/(?P<args>.*)/)?"
```

It may seem scary at first, but it's pretty easy to change. For example, let's say you want to start passive matches with `.` instead of `:`, you can write:

```yaml
passive_match_regex: "(?P<name>.\\p{L}+)(/(?P<args>.*)/)?"
```

Notice the `.` after `<name>` instead of the `:`.

Another thing you may want to change are the external argument separators, let's say you want to use parenthesis `()` instead of the default `//`. A solution would be:

```yaml
passive_match_regex: "(?P<name>:\\p{L}+)(\\((?P<args>.*)\\))?"
```

Notice the `\\(` and `\\)` difference before and after the `<args>` cell. 

A thing to keep in mind here is that, although you changed the external argument char, you 
didn't change the **argument delimiter**, and therefore you still need to write `:trigger(arg1/arg2)`. To solve the problem, you have to change the following parameter:

#### `passive_arg_delimiter`

Let's say you want to separate inner arguments by a comma `,`, such as `:trigger/arg1,arg2/`. You can do so by customizing the `passive_arg_delimiter` param:

```yaml
passive_arg_delimiter: ","
```

An important thing to keep in mind here is **escaping**: what if one of the arguments contains the arg delimiter? 

By default, you can escape the character with `\`,  such as `:trigger/Today is the 10\/12/`, but you can also change this escaping char by using the following parameter:

#### `passive_arg_escape`

This option regulates which character will act as an escape, by default is `\`.