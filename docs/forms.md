---
title: Forms 
layout: docs
---

Since version 0.7.0, espanso is capable of creating arbirarly complex input forms:

![Espanso Form](/assets/images/macform.png)

These open up a world of possibilities, allowing the user to create matches with many arguments, as well as injecting those values into custom Scripts or Shell commands.

### Prerequisites

In order to use forms, espanso requires [modulo](https://github.com/federico-terzi/modulo) to be installed. 
If you are a Windows or macOS user, then modulo is most likely already installed with recent versions of espanso, but if you are using Linux, you'll need to [install it manually](/install/linux/#installing-modulo).

### Getting started

Let's say you want to create a match for birthday greetings, personalized with the person name. With forms, that would translate to:

{% raw %}
```yaml
  - trigger: ":greet"
    form: |
      Hey {{name}},
      Happy Birthday!
```
{% endraw %}

Then, after typing `:greet`, you will be prompted with:

![Form example](/assets/images/form1.png)

After entering the desired name, we can submit the form either by clicking "Submit" or pressing CTRL+Enter.

The key here is to specify the `form` field rather than `replace`, which is a shorthand for the verbose syntax explained in the following sections.

{% raw %}
You can create as many fields as you like, just indicate them with the double-brackets `{{field_name}}` syntax.
{% endraw %}

### Controls

In the previous example, we've seen how to use simple text fields, but `modulo` supports many controls, such as:

* Multiline text field
* Choice box
* List box

In order to use another control, we need to specify it in the `form_fields` parameter. Let's say we want to add a multiline text field to our previous example to write our custom message inside the greetings:

{% raw %}
```yaml
  - trigger: ":greet"
    form: |
      Hey {{name}},
      {{text}}
      Happy Birthday!
    form_fields:
      text:
        multiline: true
```
{% endraw %}

After saving and triggering the match, we would be prompted with a form like the following:

![Form example](/assets/images/form2.png)

Let's analyze the most important bits:

{% raw %}
1. Inside the `form` parameter we specified the `{{text}}` field. This name is arbitrary, you can put whatever you want there.
2. In the `form_fields` parameter, we specified that the `text` field had property `multiline: true`
{% endraw %}

Each control has its own options, so let's see them separately:

#### Text Fields

{% raw %}
Text Fields are the default control. Anytime you specify a new field using the `{{field_name}}` syntax, that field is considered a text field, if not specified otherwise.

| Property | Description | Default value |
| ---------|-------------|---------------|
| multiline | If `true`, the text field becomes a multiline text area | `false` |
| default | Specify the default value of the field | `null` |

{% endraw %}

#### Choice Box

Choice boxes are fields in which the user can select one choice in a list. In order to use it, you have to specify the `type: choice` parameter, along with an array of `values`:

{% raw %}
```yaml
  - trigger: ":form"
    form: |
      {{choices}}
    form_fields:
      choices:
        type: choice
        values:
          - First choice
          - Second choice
```
{% endraw %}

Which produces:

![Form example](/assets/images/form3.png)

| Property | Description | Default value |
| ---------|-------------|---------------|
| default | Specify the default value of the field | `null` |

#### List Box

List boxes are completely equivalent to Choice Boxes, with the only difference of requiring `type: list` rather than `type: choice`.

{% raw %}
```yaml
  - trigger: ":form"
    form: |
      {{choices}}
    form_fields:
      choices:
        type: list
        values:
          - First choice
          - Second choice
```
{% endraw %}

Which produces:

![Form example](/assets/images/form4.png)

### Using with Script and Shell extension

The syntax proposed above works for most cases, but there are times in which you might want to harness the full power of espanso forms, for example using them with the Script and Shell extension.

The first important thing to understand is that the following syntax:

{% raw %}
```yaml
  - trigger: ":form"
    form: "Hey {{name}}, how are you?"
```

is a shorthand of the following match:

```yaml
  - trigger: ":form"
    replace: "Hey {{form1.name}}, how are you?"
    vars:
      - name: "form1"
        type: form
        params:
          layout: "Hey {{name}}, how are you?"
```

What this does is simply generating a form with the given layout, and then injecting the resulting fields (`form1.name`) into the replacement text. It should be clear now that **forms are extensions themselves**, just like the Date and Script extension.

All right, but **how can we use forms with the shell extension**? 

Let's say that we want to create a match that prompts for user input, and then expands to the reverse of what the user inserted.
That could be implemented with:

```yaml
  - trigger: ":rev"
    replace: "{{reversed}}"
    vars:
    - name: form1
      type: form
      params:
        layout: |
          Reverse {{name}}
    - name: reversed
      type: shell
      params:
       cmd: "echo $ESPANSO_FORM1_NAME | rev"
```

The key aspect here is that the value of the form field is injected in the shell variable as `ESPANSO_FORM1_NAME`. The naming is pretty straightforward, as the form variable is called `form1` and the field is called `name`.

To understand more about the variable injection mechanism, please read the [Advanced Topics](/docs/matches/#advanced-topics) section.
{% endraw %}

#### Controls with the Verbose syntax

If you want to use other Controls in a form specified using the above (verbose) syntax, you would have to specify the `fields` parameter:

{% raw %}
```yaml
  - trigger: ":form"
    replace: "Hey {{form1.name}}, how are you? Do you like {{form1.fruit}}?"
    vars:
      - name: "form1"
        type: form
        params:
          layout: "Name: {{name}} \nFruit: {{fruit}}"
          fields:
            name:
              multiline: true
            fruit:
              type: list # or `choice`
              values:
                - Apples
                - Bananas
                - Oranges
                - Peaches
```
{% endraw %}

Note that **the `fields` parameter content is specified as the `form_fields` parameter explaned above**.

### macOS remarks

For the expansion to take place on macOS, you have to release the submit keys (CTRL+Enter) after submitting the form.

#### Enabling Tab traversal

By default, on macOS the Tab key can only be used to change focus between Text fields. To make forms completely navigable using the Tab key (including Lists, Selects and Buttons), you have to enable it with these steps:

1. Open *System Preferences*
2. Navigate to the Keyboard panel
3. Open the *Keyboard Shortcuts* tab
4. Near the bottom of the dialog, select "All controls".

Then you should be able to use the Tab key to navigate between form controls.