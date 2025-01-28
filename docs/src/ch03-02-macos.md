# macOS

After installing the prerequisites, you are ready to build Espanso on macOS.

Espanso supports two targets on macOS: plain executable and App Bundle. For most cases, the App Bundle format is preferrable.

#### App Bundle

You can build the App Bundle by running:

```bash
cargo make --profile release -- create-bundle
```

This will create the `Espanso.app` bundle in the `target/mac` directory.


