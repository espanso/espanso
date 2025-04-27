# How to develop the schema

Hi fellow coder! I will guide you here how to develop the schemas.

Schemas are just a colections of types, or geometric forms the data needs to be
compliant.

Both schemas can be used in jsons, not only in yamls. The way of telling VS Code
(and other editors) to use a schema is by the `$schema` key. For example:

```json
{
    "$schema": "http://json-schema.org/draft-04/schema#"
}
```

Now, you don't need a schema loaded in the web to use it, it can load a local
file in your computer, like:

```json
{
    "$schema": "./match.schema.json",
}
```

So, if you create a file in this folder, say `espanso-repo-folder/schemas/matches.json`
and you add the `$schema` key, you can develop in you local computer while you
edit the `match.schema.json` file.

Take in mind that the `.schema.` section of the name it's just a convention, it
has nothing to do with the schema.

### Where can I read more about JSON schemas?

In the [json schema website](https://json-schema.org/).

And you can use more schemas in your files in the [JSON Schema store](https://www.schemastore.org/json/)
