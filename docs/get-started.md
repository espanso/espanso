---
title: Getting Started
layout: docs
---
In this section we will cover the basics of espanso to get you started immediately.
Make sure to [install espanso](/install) before diving into the next sections.

### Starting espanso

If you followed the [Windows](/install/win/) or [MacOS](/install/mac/) installation correctly, **espanso will be automatically started** when you power up your computer. There are times, however, when you may need to start
espanso explicitly, such as when you're using Linux.

It's very easy to check if espanso is currently running: if you're using **MacOS** or **Windows**, you should see
the **icon in the status bar**. If you don't see it, or if you're using **Linux**, another way to check it is to **open a terminal** and type:

```
espanso status
```

If you see "`espanso is not running`", then you'll need to start espanso manually with the following command:

```
espanso start
```

At this point you are ready to use espanso. Open any typing application and type `:espanso`, you should
see `Hi there!` appear. 

If you don't see it, make sure espanso is currently running. You could also try to repeat the installation procedure.

### Understanding Matches

espanso works by **detecting** your keypresses and **replacing** them when they match a specific keyword, called *trigger*.

![How espanso works](/assets/images/match1.png)

The rule that associate the *trigger* with the *replaced text* is called **Match** and is a core concept of espanso.
Matches are very flexible and powerful to solve many tasks. You can learn all about Matches in their [documentation](/docs/matches/) page.

![Match](/assets/images/match2.png)

espanso ships with very few built-in Matches to give you the maximum flexibility, but you can expand it's capabilities
in two ways: creating your own **custom matches** or **installing packages**. Both of these possibilities are introduced below.

### Creating your own Match

### Installing a Package