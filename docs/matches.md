---
title: Matches
layout: docs
---
Matches are the espanso's core component and define the substitutions that will take place.

### Static Matches

In their most basic form, **Matches are pairs that associate a *trigger* with a *replaced text***.

For example, we can define a match that will expand every occurrence of `hello` with `world` while we are typing. Using the [YAML](https://en.wikipedia.org/wiki/YAML) syntax, it can be expressed as:

```
- trigger: "hello"
  replace: "world"
```

To replace the original text with a *multi-line* expansion, we can use the `\n` line terminator character, such as:

```
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
```
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

TODO

### Script Extension

### Shell Extension

### Date Extension