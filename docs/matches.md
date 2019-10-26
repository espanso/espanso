---
title: Matches
layout: docs
---
Matches are the espanso's core component and define the substitutions that will take place.

### Static Matches

In their most basic form, **Matches are pairs that associate a *trigger* with a *replaced text***.

For example, we can define a match that will expand every occurrence of `hello` with `world` while we are typing. Using the [YAML](https://en.wikipedia.org/wiki/YAML) syntax, it can be expressed as:

```yml
- trigger: "hello"
  replace: "world"
```

To replace the original text with a *multi-line* expansion, we can use the `\n` line terminator character, such as:

```yml
- trigger: "hello"
  replace: "line1\nline2"
```

These kind of expansions are simple text replacements and are *static*.

### Dynamic Matches

Static matches are suitable for many tasks, but can be problematic when we need an **expansion that changes dynamically**. For those situations, espanso introduces the concepts of **variables** and **extensions**.

**Variables** can be used in the **replace** clause of a Match to include the *output* of a dynamic component, the **extension**. To make things more clear, let's see an example:

We want to create a match that, everytime we type `:now`, it expands it to include the current time, like:

```
It's 11:29
```

Let's add the following match to a configuration file, such as the `default.yml` config.

{% raw %}
```yaml
- trigger: ":now"
  replace: "It's {{mytime}}"
  vars:
    - name: mytime
      type: date
      params:
        format: "%H:%M"
```
{% endraw %}

And restart espanso with:

```
espanso restart
```

At this point, everytime we type `:now`, we should see something like: `It's 09:33`!

Let's analyze the match step by step:

```yml
- trigger: ":now"
```

In the first line we declare the trigger `:now`, that must be typed by the user to expand the match.

{% raw %}
```yml
  replace: "It's {{mytime}}"
```
{% endraw %}

In the second line, we declare the *replace text* as usual, but this time we include the `mytime` **variable**,
that will contain the output of the **extension** used below.

{% raw %}
```yml
  vars:
    - name: mytime
      type: date
```
{% endraw %}

In the next lines, we defined the `mytime` variable as type **date**. The type of a variable defines
the **extension** that will be executed to calculate its value. In this case, we use the [Date Extension](#date-extension).

```yml
      params:
        format: "%H:%M"
```

In the remaining lines we declared the **parameters** used by the extension, in this case the *date format*.

### Cursor Hints

Let's say you want to use espanso to expand some HTML code snippets, such as:

```yaml
  - trigger: ":div"
    replace: "<div></div>"
```

With this match, any time you type `:div` you get the `<div></div>` expansion, with the cursor at the end.

While being useful, this snippet would have been much more convenient if **the cursor was positioned
between the tags**, such as `<div>|</div>`.

Starting from version 0.3.2, espanso supports **cursor hints**, a way to control the position of the cursor
after the expansion. 

Using them is very simple, just insert `$|$` where you want the cursor to be positioned, in this case:

```yaml
  - trigger: ":div"
    replace: "<div>$|$</div>"
```

If you now type `:div`, you get the `<div></div>` expansion, with the cursor between the tags!

#### Things to keep in mind

* You can only define **one cursor hint** per match. Multiple hints will be ignored.
* This feature should be used with care in **multiline** expansions, as it may yield
  **unexpected results** when using it in code editors that support **auto indenting**. 
  This is due to the way the feature is implemented: espanso simulates a series of `left arrow`
  key-presses to position the cursor in the right position. This works perfectly in single line
  replacements or in non-autoindenting fields, but can be problematic in code editors, as they
  automatically insert indentations that modify the number of required presses in a way
  espanso is not capable to detect.

### Word Triggers

If you ever thought about using espanso as an **autocorrection tool for typos**, you may have experienced
this problem:

Let's say you occasionally type `ther` instead of `there`. Before the introduction of *word triggers*, 
you could have used espanso like this:

```yaml
  - trigger: "ther"
    replace: "there"
```

This would correctly replace `ther` with `there`, but it also has the problem of expanding
`other` into `othere`, making it unusable.

With *word triggers* you can now add the `word: True` property to a match, telling espanso
to only trigger that match if surrounded by *word separators* ( such as *spaces*, *commas* and *newlines*). 
So in this case it becomes:

```yaml
  - trigger: "ther"
    replace: "there"
    word: True
```

At this point, espanso will only expand `ther` into `there` when used as a standalone word.
For instance:

Before | After |
--- | ---
Is ther anyone else? | Is there anyone else? | `ther` is converted to `there`
I have other interests | I have other interests | `other` is left unchanged

### Script Extension

There will be tasks for which espanso was not designed for. For those cases, espanso offers the
**Script Extension**, that enables you to call an **external script**, written in **any language**,
 and use its output in a match.

To better understand this feature, let's dive into an example:

We want to expand a match into the output of a **Python** script. Let's create the `script.py` file,
place it anywhere you want and paste the following code:

```python
print("Hello from python")
```

Now take note of the **path** of the script, and add the following match to the espanso configuration:

{% raw %}
```yaml
- trigger: ":pyscript"
  replace: "{{output}}"
  vars:
    - name: output
      type: script
      params:
        args:
          - python
          - /path/to/your/script.py
```
{% endraw %}

As always, restart espanso with `espanso restart`. 
If you now try to type `:pyscript` anywhere, you should see `Hello from python` appear.

You can do the same thing with any programming language, just change the `args` array accordingly.

#### A note on performance

Because of the execution time, you should limit yourself to fast-running scripts to avoid
any lag.

### Shell Extension

The **Shell Extension** is similar to the [Script Extension](#script-extension), but instead of executing
a script, it executes **shell commands**. This offers a lot of flexibility on Unix systems thanks to the
`bash` shell.

Let's say you regularly send your IP address to your coworkers. You can setup a match to fetch your public
IP from [ipify](https://www.ipify.org/).

> Note: this example uses the `curl` command, usually preinstalled in most Unix systems.

{% raw %}
```yml
- trigger: ":ip"
  replace: "{{output}}"
  vars:
    - name: output
      type: shell
      params:
        cmd: "curl 'https://api.ipify.org'"
```
{% endraw %}

As always, restart espanso with `espanso restart`. Now everytime you type `:ip`, it gets expanded to your public
IP address!

#### Bash pipes

This extension also supports bash **pipes** as your shell, such as:

{% raw %}
```yml
- trigger: ":localip"
  replace: "{{output}}"
  vars:
    - name: output
      type: shell
      params:
        cmd: "ip a | grep 'inet 192' | awk '{ print $2 }'"
```
{% endraw %}

### Date Extension

The **Date Extension** can be used to include *date* and *time* information in a match. 

The most important aspect to consider when using this extension is the `format` parameter,
that specifies how the date will be rendered. A **list of all the possible options** can be
found in the [official chrono documentation](https://docs.rs/chrono/0.3.1/chrono/format/strftime/index.html).

{% raw %}
```yaml
- trigger: ":now"
  replace: "It's {{mytime}}"
  vars:
    - name: mytime
      type: date
      params:
        format: "%H:%M"
```
{% endraw %}