# macOS

After installing the prerequisites, you are ready to build Espanso on macOS.

Espanso supports two targets on macOS: plain executable and App Bundle. For most cases, the App Bundle format is preferrable.

#### App Bundle

You can build the App Bundle by running:

```console
cargo build --no-default-features --features modulo,native-tls --release
bash ./scripts/create_bundle.sh
```

This will create the `Espanso.app` bundle in the `target/mac` directory as a universal binary that will run well on both Intel and Apple Silicon machines.
