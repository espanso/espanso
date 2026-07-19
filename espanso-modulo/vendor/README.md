# Vendored version of wxWidgets

We use wxWidgets to build the UI of espanso. In the past it was decided to 
vendor the dependencies and 3.1.5 was the initial version.

After August 31, 2025 the CI started to fail. I don't know exactly what happened
but when you build this it errors this 4 times:

```console
  /Users/macosuser/other-repos/espanso/espanso/target/debug/build/espanso-modulo-ba0d1ed7d16f3ee7/out/wx/src/png/pngpriv.h:536:16: fatal error: 'fp.h' file not found
    536 | #      include <fp.h>
        |                ^~~~~~
```

For this reason, I did a patch you can apply to generate the
`wxWidgets-3.1.5-patched.zip` version.

## how to build the zip file

1) download `wxWidgets-3.1.5.zip` from github

2) copy it to this `vendor/` folder

3) extract it

4) apply this patch contained in the folder

```bash
cp apply.patch wxWidgets-3.1.5/
cd wxWidgets-3.1.5/
git apply apply.patch
```

5) zip it again and it's done!
