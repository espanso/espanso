# Dev recipes and cookbook

Developing a project is sometimes complex, and we use these tools every now and 
then to check if we are doing everything right. These aren't tools that we 
_need_ to run on CI, but it's nice to have them and check if it's allright.

## `cargo udeps` to find unused dependencies

install it by

```bash
# traditional cargo install
cargo install cargo-udeps --locked
# or with cargo binstall
cargo binstall cargo-udeps --locked
```

Nightly toolchain is needed, just for this tool only

```bash
rustup toolchain install nightly
```

and then run:

```bash
cargo +nightly udeps --all-targets
```

There is a [sample issue here](https://github.com/espanso/espanso/issues/1833),
and [its PR there](https://github.com/espanso/espanso/pull/1856)
We know some false positives, running on windows or linux gives:

```bash
unused dependencies:
`espanso-mac-utils v0.1.0 (C:\Users\user\repos\espanso\espanso-mac-utils)`
├─── dependencies
│    └─── "regex"
└─── build-dependencies
     └─── "cc"
`espanso-modulo v0.1.0 (C:\Users\user\repos\espanso\espanso-modulo)`
└─── build-dependencies
     └─── "glob"
Note: They might be false-positive.
      For example, `cargo-udeps` cannot detect usage of crates that are only used in doc-tests.
      To ignore some dependencies, write `package.metadata.cargo-udeps.ignore` in Cargo.toml.
```
These are because the windows and linux can't see through the `target = macos`

## check for outdated dependencies

install it by
```bash
# traditional cargo install
cargo install cargo-outdated --locked
# or with cargo binstall
cargo binstall cargo-outdated --locked
```
An run it with
```bash
cargo outdated
```

## make a coverage report

install `cargo-tarpaulin`

```
cargo binstall cargo-tarpaulin
```

and then

```
cargo tarpaulin
```

that reads the file `tarpaulin.toml` where we have the config. Once it's
finished, it makes an HTML report in the root folder (gitignored)

You can also `Ctrl + Shift + p` in vs code and select 
`Tasks: Run Task: rust coverage`

## Cargo mutants

[espanso-mutants](https://github.com/espanso/espanso-mutants) is a repository
where it holds the result of `cargo-mutants`, a mutational testing library.

You can find more about what mutation testing is [on this blogpost](https://notashelf.dev/posts/on-mutation-testing)

We would like to fix our tests, so it kills any living mutant left, and we have
~3800 of them!

## format everything

We added `cargo make fmt` to format everything, it requires some tools:

- rust is formatted with `cargo fmt`. Easy.
- json is formatted with [biome](https://next.biomejs.dev/guides/getting-started/)
You can install biome in your node package manager globally like this:

```bash
# node
npm install --global @biomejs/biome
# bun
bun install --global @biomejs/biome
```

- and `clang-format` to format files

You can install it in Windows with npm (so you don't need to deal with MinGW)

```
npm install --global clang-format
# or with bun
bun install --global clang-format
```

In linux, it will come with your package manager.

and in macOS:
```zsh
brew install clang-format
```

- if you are in macOS or linux, we add `nix` to the list: [Download it](https://nixos.org/download/)

## check for nested dependencies

This script was written by n8henrie, and counts how many times you use a 
dependency (if you don't have it on the root Cargo.toml).

```python
# parse_deps.py
# needs python >= 3.11
"""Quick script to find and parse all dependency versions in the workspace."""

import tomllib
from pathlib import Path
from collections import Counter
from pprint import pprint

cargo_tomls = list(p for p in Path(".").glob("**/Cargo.toml") if len(p.parents) > 1)
counter = Counter(
    (name, str(version)) for  p in cargo_tomls for name, version in tomllib.loads(p.read_text())["dependencies"].items() if not ("path" in version or "workspace" in version))

if __name__ == "__main__":
    pprint(counter)
```

and you can run it with:

```console
# if you use uv
uv run parse_deps.py
```

## Benchmark script
The script was written by [bladeacer](https://github.com/bladeacer), it
benchmarks the run times of the various command line flags of the espanso binary.

Ensure that espanso, [hyperfine](https://github.com/sharkdp/hyperfine) and [Python](https://python.org/downloads)
has been installed before running the script.
> For Linux or Mac users, it is recommended to use your package manager.

```python
#!/usr/bin/env python

import shutil
import subprocess
import sys
import shlex

def run_hyperfine_benchmark(commands, **kwargs):
    """
    Checks for the 'hyperfine' binary and runs a benchmark if found.
    """
    hyperfine_path = shutil.which('hyperfine')
    
    if hyperfine_path:
        benchmark_name = kwargs.get('name', 'Benchmark Set')
        print(f"\n--- {benchmark_name} Benchmark Start ---")
        
        cmd = [hyperfine_path]
        
        if kwargs.get('warmup'):
            cmd.extend(['--warmup', str(kwargs['warmup'])])
        if kwargs.get('runs'):
            cmd.extend(['--runs', str(kwargs['runs'])])

        if kwargs.get('export_json'):
            cmd.extend(['--export-json', str(kwargs['export_json'])])
        if kwargs.get('export_markdown'):
            cmd.extend(['--export-markdown', str(kwargs['export_markdown'])])
        if kwargs.get('i'):
            cmd.append('--ignore-failure')
            
        quoted_commands = [shlex.quote(c) for c in commands]
        cmd.extend(quoted_commands)

        print(f"Executing command: {' '.join(cmd)}")
        
        try:
            result = subprocess.run(cmd, check=True, text=True, capture_output=False) 
            
            print("-----------------------------------")
            print(f"{benchmark_name} finished successfully.")
            
        except subprocess.CalledProcessError as e:
            print(f"Error during {benchmark_name} execution (non-zero exit code {e.returncode}):", file=sys.stderr)
            sys.exit(1)
        except OSError as e:
            print(f"An OS error occurred during execution: {e}", file=sys.stderr)
            sys.exit(1)
        
        print(f"--- {benchmark_name} End ---\n")

    else:
        print("Warning: 'hyperfine' is not installed or not found in your system's PATH.", file=sys.stderr)
        print("Please install it to run the benchmarks. Exiting.", file=sys.stderr)
        sys.exit(1)

commands_set = [
    'espanso',
    'espanso cmd',
    'espanso cmd enable',
    'espanso cmd disable',
    'espanso cmd toggle',
    'espanso cmd search',
    'espanso edit',
    'espanso env-path',
    'espanso env-path unregister',
    'espanso env-path register',
    'espanso help',
    'espanso log',
    'espanso match',
    'espanso package',
    'espanso package list',
    'espanso path packages',
    'espanso install',
    'espanso install lorem',
    'espanso package install lorem',
    'espanso package uninstall lorem',
    'espanso package update all',
    'espanso package install',
    'espanso package uninstall',
    'espanso package update',
    'espanso path',
    'espanso path config',
    'espanso path default',
    'espanso path packages',
    'espanso path runtime',
    'espanso service',
    'espanso service check',
    'espanso service unregister',
    'espanso service register',
    'espanso service restart',
    'espanso service status',
    'espanso service stop',
    'espanso service start',
    'espanso service start --unmanaged',
    'espanso workaround',
    'espanso workaround secure-input',
    'espanso status',
    'espanso start',
    'espanso start --unmanaged',
    'espanso restart',
    'espanso match list',
    'espanso match list -j',
    'espanso match list -t',
    'espanso match list -n',
    'espanso match exec',
    'espanso match exec -t :espanso',
    'espanso match exec -t :date',
    'espanso match exec -t :shell'
]

hyperfine_args = {
    'name': 'Standard',
    'warmup': 2,
    'runs': 50,
    'i': True,
    'export_markdown': "results.md",
    'export_json': "results.json"
}


if __name__ == "__main__":
    run_hyperfine_benchmark(commands_set, **hyperfine_args)
```

#### Running the benchmark script
Name and save the script as `benchmarks.py`.

Set file permissions if you are on Linux or MacOS.
```bash
sudo chmod +x ./benchmarks.py
```

Then call the script with the following
```bash
./benchmarks.py
```

Of if you are on Windows
```bash
python ./benchmarks.py
```

The script saves the benchmark results to `results.md` and `results.json`.

#### Benchmark script: example output
```md
| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `espanso` | 529.5 ± 73.2 | 410.6 | 675.0 | 6679.68 ± 15608.68 |
| `'espanso cmd'` | 4.4 ± 4.3 | 0.0 | 14.8 | 55.17 ± 139.56 |
| `'espanso cmd enable'` | 8.6 ± 4.6 | 0.0 | 16.3 | 108.08 ± 258.72 |

...
```

```json
{
  "results": [
    {
      "command": "espanso",
      "mean": 0.5294862457400001,
      "stddev": 0.0731799752802901,
      "median": 0.50886927814,
      "user": 0.041466739999999974,
      "system": 0.0514393,
      "min": 0.41057140914000007,
      "max": 0.67504138914,
      "times": [...]
    },
    ...
  ]
}
```

#### Benchmark script: analysing results
The JSON output is more useful for statistical analysis. The hyperfine
repository has [scripts for this purpose](https://github.com/sharkdp/hyperfine/tree/master/scripts).
