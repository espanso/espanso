/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::{
    fs,
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, read::DecoderReader, write::EncoderWriter};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use tar::{Archive, Builder, EntryType};
use walkdir::WalkDir;

use super::{CliModule, CliModuleArgs, PathsOverrides};
use crate::{cli::util::CommandExt, error_eprintln, path::Paths, util::set_command_flags};

struct WhitespaceFilteringReader<R> {
    inner: R,
}

impl<R> WhitespaceFilteringReader<R> {
    fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl<R: Read> Read for WhitespaceFilteringReader<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }

        let mut total = 0;
        let mut buf = [0u8; 8192];
        while total == 0 {
            let count = self.inner.read(&mut buf)?;
            if count == 0 {
                return Ok(0);
            }

            for &byte in &buf[..count] {
                if byte.is_ascii_whitespace() {
                    continue;
                }
                out[total] = byte;
                total += 1;
                if total == out.len() {
                    break;
                }
            }
        }

        Ok(total)
    }
}

pub fn new_export() -> CliModule {
    CliModule {
        requires_paths: true,
        subcommand: "export".to_string(),
        entry: export_main,
        ..Default::default()
    }
}

pub fn new_import() -> CliModule {
    CliModule {
        requires_paths: true,
        subcommand: "import".to_string(),
        entry: import_main,
        ..Default::default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Scope {
    Config,
    Matches,
    Packages,
}

impl Scope {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "config" => Some(Scope::Config),
            "matches" | "match" => Some(Scope::Matches),
            "packages" => Some(Scope::Packages),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ScopeSelection {
    config: bool,
    matches: bool,
    packages: bool,
}

impl ScopeSelection {
    fn all() -> Self {
        Self {
            config: true,
            matches: true,
            packages: true,
        }
    }

    fn is_empty(self) -> bool {
        !self.config && !self.matches && !self.packages
    }

    fn includes(self, scope: Scope) -> bool {
        match scope {
            Scope::Config => self.config,
            Scope::Matches => self.matches,
            Scope::Packages => self.packages,
        }
    }
}

fn export_main(args: CliModuleArgs) -> i32 {
    let paths = args.paths.expect("missing paths argument");
    let cli_args = args.cli_args.expect("missing cli_args argument");
    let sub_args = &cli_args;

    let scope_selection = match parse_scope_selection(sub_args.value_of("scope")) {
        Ok(scope) => scope,
        Err(err) => {
            error_eprintln!("invalid scope: {err}");
            return 1;
        }
    };

    if let Err(err) = export_payload_to_stdout(&paths, scope_selection) {
        error_eprintln!("unable to export: {err}");
        return 1;
    }

    0
}

fn import_main(args: CliModuleArgs) -> i32 {
    let paths = args.paths.expect("missing paths argument");
    let paths_overrides = args.paths_overrides.expect("missing paths_overrides");
    let cli_args = args.cli_args.expect("missing cli_args argument");
    let sub_args = &cli_args;

    let scope_selection = match parse_scope_selection(sub_args.value_of("scope")) {
        Ok(scope) => scope,
        Err(err) => {
            error_eprintln!("invalid scope: {err}");
            return 1;
        }
    };

    let convert_lb = sub_args.is_present("convert-lb");
    let skip_confirmation = sub_args.is_present("yes");

    if let Err(err) = confirm_import(skip_confirmation) {
        error_eprintln!("{err}");
        return 1;
    }

    if let Err(err) =
        import_payload_from_stdin(&paths, &paths_overrides, scope_selection, convert_lb)
    {
        error_eprintln!("unable to import: {err}");
        return 1;
    }

    0
}

fn parse_scope_selection(value: Option<&str>) -> Result<ScopeSelection> {
    let value = match value {
        Some(value) => value.trim(),
        None => return Ok(ScopeSelection::all()),
    };

    if value.is_empty() {
        bail!("scope cannot be empty");
    }

    let mut selection = ScopeSelection::default();
    for raw in value.split(',') {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let scope = Scope::from_str(trimmed).with_context(|| {
            format!("unknown scope '{trimmed}', expected config,matches,packages")
        })?;
        match scope {
            Scope::Config => selection.config = true,
            Scope::Matches => selection.matches = true,
            Scope::Packages => selection.packages = true,
        }
    }

    if selection.is_empty() {
        bail!("no valid scopes provided");
    }

    Ok(selection)
}

fn export_payload_to_stdout(paths: &Paths, selection: ScopeSelection) -> Result<()> {
    let stdout = io::stdout();
    let handle = stdout.lock();
    let encoder = EncoderWriter::new(handle, &STANDARD);
    let mut gzip = GzEncoder::new(encoder, Compression::default());
    let mut builder = Builder::new(&mut gzip);

    let matches_dir = paths.config.join("match");
    let packages_dir = paths.packages.clone();
    let matches_skip_packages =
        matches_dir.is_dir() && packages_dir.is_dir() && packages_dir.starts_with(&matches_dir);

    if selection.config {
        let config_dir = paths.config.join("config");
        append_dir_to_archive(&mut builder, &config_dir, Path::new("config"), None)?;
    }
    if selection.matches {
        let matches_dir = paths.config.join("match");
        let skip_dir = if matches_skip_packages {
            Some(packages_dir.as_path())
        } else {
            None
        };
        append_dir_to_archive(&mut builder, &matches_dir, Path::new("matches"), skip_dir)?;
    }
    if selection.packages {
        append_dir_to_archive(&mut builder, &packages_dir, Path::new("packages"), None)?;
    }

    builder.finish()?;
    drop(builder);
    let mut encoder = gzip.finish()?;
    let mut handle = encoder.finish()?;
    handle.write_all(b"\n")?;
    Ok(())
}

fn append_dir_to_archive<W: Write>(
    builder: &mut Builder<W>,
    source_dir: &Path,
    archive_prefix: &Path,
    skip_dir: Option<&Path>,
) -> Result<()> {
    if !source_dir.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(source_dir).follow_links(false) {
        let entry = entry?;
        let path = entry.path();

        if let Some(skip) = skip_dir {
            if path.starts_with(skip) {
                continue;
            }
        }

        let relative = path
            .strip_prefix(source_dir)
            .context("unable to compute relative archive path")?;

        let archive_path = if relative.as_os_str().is_empty() {
            archive_prefix.to_path_buf()
        } else {
            archive_prefix.join(relative)
        };

        if entry.file_type().is_dir() {
            builder.append_dir(&archive_path, path)?;
        } else if entry.file_type().is_file() {
            builder.append_path_with_name(path, &archive_path)?;
        } else {
            // Preserve other entry types (like symlinks) as-is.
            builder.append_path_with_name(path, &archive_path)?;
        }
    }

    Ok(())
}

fn confirm_import(skip_confirmation: bool) -> Result<()> {
    if skip_confirmation {
        return Ok(());
    }

    let prompt =
        "This will DELETE your existing Espanso data for the selected scope(s) and replace it. Continue? [y/N]";
    println!("{prompt}");

    let Some(reader) = open_confirmation_reader()? else {
        bail!("unable to read confirmation prompt");
    };

    confirm_import_from_reader(reader)
}

fn open_confirmation_reader() -> Result<Option<Box<dyn Read>>> {
    #[cfg(unix)]
    {
        if let Ok(file) = fs::File::open("/dev/tty") {
            return Ok(Some(Box::new(file)));
        }
    }

    #[cfg(windows)]
    {
        if let Ok(file) = fs::File::open("CONIN$") {
            return Ok(Some(Box::new(file)));
        }

        // If we started without a console, try attaching to the parent console and retry.
        let _ = crate::util::attach_console();
        if let Ok(file) = fs::File::open("CONIN$") {
            return Ok(Some(Box::new(file)));
        }
    }

    Ok(None)
}

fn confirm_import_from_reader<R: Read>(reader: R) -> Result<()> {
    let mut reader = io::BufReader::new(reader);
    let mut buf = [0u8; 1];
    loop {
        let count = reader.read(&mut buf)?;
        if count == 0 {
            bail!("unable to read confirmation prompt");
        }
        let byte = buf[0];
        if matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
            continue;
        }
        if matches!(byte, b'y' | b'Y') {
            return Ok(());
        }
        if matches!(byte, b'n' | b'N') {
            bail!("aborted by user");
        }
        bail!("invalid confirmation input");
    }
}

fn import_payload_from_stdin(
    paths: &Paths,
    paths_overrides: &PathsOverrides,
    selection: ScopeSelection,
    convert_lb: bool,
) -> Result<()> {
    let stdin = io::stdin();
    let handle = stdin.lock();
    let filtered = WhitespaceFilteringReader::new(handle);
    let decoder = DecoderReader::new(filtered, &STANDARD);
    let gzip = GzDecoder::new(decoder);
    let mut archive = Archive::new(gzip);

    prepare_import_targets(paths, selection)?;

    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?.to_path_buf();
        let normalized = sanitize_relative_path(&entry_path)?;
        let (scope, scope_relative) = split_scope_path(&normalized)?;

        if !selection.includes(scope) {
            continue;
        }

        let target_root = scope_root(paths, scope);
        let target_path = target_root.join(&scope_relative);
        ensure_inside_root(&target_root, &target_path)?;

        match entry.header().entry_type() {
            EntryType::Directory => {
                fs::create_dir_all(&target_path)?;
            }
            EntryType::Regular => {
                let mut data = Vec::new();
                entry.read_to_end(&mut data)?;
                let data = maybe_convert_line_breaks(&target_path, data, convert_lb)?;
                write_atomic(&target_path, &data)?;
            }
            _ => {
                bail!(
                    "unsupported archive entry type for {}",
                    entry_path.display()
                );
            }
        }
    }

    restart_espanso(paths_overrides)?;
    Ok(())
}

fn prepare_import_targets(paths: &Paths, selection: ScopeSelection) -> Result<()> {
    if selection.config {
        let config_dir = paths.config.join("config");
        delete_tree(&config_dir)?;
        fs::create_dir_all(&config_dir)?;
    }

    if selection.matches {
        let matches_dir = paths.config.join("match");
        let packages_dir = paths.packages.clone();
        if packages_dir.starts_with(&matches_dir) && !selection.packages {
            clear_directory_except(&matches_dir, Some(&packages_dir))?;
        } else {
            delete_tree(&matches_dir)?;
            fs::create_dir_all(&matches_dir)?;
        }
    }

    if selection.packages {
        let packages_dir = paths.packages.clone();
        delete_tree(&packages_dir)?;
        fs::create_dir_all(&packages_dir)?;
    }

    Ok(())
}

fn delete_tree(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_file() {
        fs::remove_file(path)?;
    } else {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn clear_directory_except(root: &Path, preserve: Option<&Path>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(root).min_depth(1).contents_first(true) {
        let entry = entry?;
        let path = entry.path();

        if let Some(preserve_path) = preserve {
            if path.starts_with(preserve_path) {
                continue;
            }
        }

        if entry.file_type().is_dir() {
            fs::remove_dir(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}

fn sanitize_relative_path(path: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => {
                bail!("absolute paths are not allowed in archives")
            }
            Component::ParentDir => bail!("path traversal is not allowed in archives"),
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
        }
    }

    if normalized.as_os_str().is_empty() {
        bail!("empty path entry in archive");
    }

    Ok(normalized)
}

fn split_scope_path(path: &Path) -> Result<(Scope, PathBuf)> {
    let mut components = path.components();
    let prefix = components
        .next()
        .context("archive entry missing scope prefix")?;

    let scope = match prefix {
        Component::Normal(name) => {
            let name = name.to_string_lossy();
            Scope::from_str(&name).with_context(|| format!("unknown scope prefix '{name}'"))?
        }
        _ => bail!("invalid scope prefix in archive entry"),
    };

    let mut remainder = PathBuf::new();
    for component in components {
        match component {
            Component::Normal(part) => remainder.push(part),
            _ => bail!("invalid archive entry path"),
        }
    }

    Ok((scope, remainder))
}

fn scope_root(paths: &Paths, scope: Scope) -> PathBuf {
    match scope {
        Scope::Config => paths.config.join("config"),
        Scope::Matches => paths.config.join("match"),
        Scope::Packages => paths.packages.clone(),
    }
}

fn ensure_inside_root(root: &Path, target: &Path) -> Result<()> {
    if !target.starts_with(root) {
        bail!("archive entry resolved outside target directory");
    }
    Ok(())
}

fn maybe_convert_line_breaks(path: &Path, data: Vec<u8>, convert: bool) -> Result<Vec<u8>> {
    if !convert || !is_yaml_path(path) {
        return Ok(data);
    }

    let text = match std::str::from_utf8(&data) {
        Ok(text) => text,
        Err(_) => {
            eprintln!(
                "warning: skipping line break conversion for non-UTF8 YAML file {}",
                path.display()
            );
            return Ok(data);
        }
    };

    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    Ok(normalized.into_bytes())
}

fn is_yaml_path(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => {
            let lower = ext.to_ascii_lowercase();
            lower == "yml" || lower == "yaml"
        }
        None => false,
    }
}

fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = temp_path_for(path);
    {
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(data)?;
        file.sync_all()?;
    }

    if path.exists() {
        let _ = fs::remove_file(path);
    }
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn temp_path_for(path: &Path) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    let extension = format!("espanso_import_tmp_{pid}_{unique}");
    path.with_extension(extension)
}

fn restart_espanso(paths_overrides: &PathsOverrides) -> Result<()> {
    let espanso_exe_path = std::env::current_exe()?;
    let mut command = Command::new(espanso_exe_path.to_string_lossy().to_string());
    command.args(["service", "restart"]);
    command.with_paths_overrides(paths_overrides);
    set_command_flags(&mut command);

    let status = command.status()?;
    if !status.success() {
        bail!("espanso restart returned a non-zero exit code");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    fn create_sample_tree(base: &Path) -> Result<Paths> {
        let config_root = base.join("config_root");
        let runtime_root = base.join("runtime");
        let packages_root = config_root.join("match").join("packages");

        fs::create_dir_all(config_root.join("config"))?;
        fs::create_dir_all(config_root.join("match"))?;
        fs::create_dir_all(&packages_root)?;

        fs::write(config_root.join("config").join("default.yml"), "config")?;
        fs::write(config_root.join("match").join("base.yml"), "matches")?;
        fs::write(packages_root.join("package.yml"), "packages")?;

        Ok(Paths {
            config: config_root,
            runtime: runtime_root,
            packages: packages_root,
            is_portable_mode: false,
        })
    }

    fn export_to_vec(paths: &Paths, selection: ScopeSelection) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        {
            let encoder = EncoderWriter::new(&mut output, &STANDARD);
            let mut gzip = GzEncoder::new(encoder, Compression::default());
            let mut builder = Builder::new(&mut gzip);

            let matches_dir = paths.config.join("match");
            let packages_dir = paths.packages.clone();
            let matches_skip_packages = matches_dir.is_dir()
                && packages_dir.is_dir()
                && packages_dir.starts_with(&matches_dir);

            if selection.config {
                append_dir_to_archive(
                    &mut builder,
                    &paths.config.join("config"),
                    Path::new("config"),
                    None,
                )?;
            }
            if selection.matches {
                let skip_dir = if matches_skip_packages {
                    Some(packages_dir.as_path())
                } else {
                    None
                };
                append_dir_to_archive(
                    &mut builder,
                    &paths.config.join("match"),
                    Path::new("matches"),
                    skip_dir,
                )?;
            }
            if selection.packages {
                append_dir_to_archive(&mut builder, &packages_dir, Path::new("packages"), None)?;
            }

            builder.finish()?;
            drop(builder);
            let mut encoder = gzip.finish()?;
            encoder.finish()?;
        }
        Ok(output)
    }

    fn import_from_bytes(
        payload: &[u8],
        paths: &Paths,
        selection: ScopeSelection,
        convert_lb: bool,
    ) -> Result<()> {
        let decoder = DecoderReader::new(payload, &STANDARD);
        let gzip = GzDecoder::new(decoder);
        let mut archive = Archive::new(gzip);

        prepare_import_targets(paths, selection)?;

        for entry in archive.entries()? {
            let mut entry = entry?;
            let entry_path = entry.path()?.to_path_buf();
            let normalized = sanitize_relative_path(&entry_path)?;
            let (scope, scope_relative) = split_scope_path(&normalized)?;

            if !selection.includes(scope) {
                continue;
            }

            let target_root = scope_root(paths, scope);
            let target_path = target_root.join(&scope_relative);
            ensure_inside_root(&target_root, &target_path)?;

            match entry.header().entry_type() {
                EntryType::Directory => {
                    fs::create_dir_all(&target_path)?;
                }
                EntryType::Regular => {
                    let mut data = Vec::new();
                    entry.read_to_end(&mut data)?;
                    let data = maybe_convert_line_breaks(&target_path, data, convert_lb)?;
                    write_atomic(&target_path, &data)?;
                }
                _ => {
                    bail!(
                        "unsupported archive entry type for {}",
                        entry_path.display()
                    );
                }
            }
        }

        Ok(())
    }

    #[test]
    fn round_trip_scopes() -> Result<()> {
        let temp = TempDir::new("espanso-offline")?;
        let paths = create_sample_tree(temp.path())?;

        let scopes = [
            ScopeSelection {
                config: true,
                matches: false,
                packages: false,
            },
            ScopeSelection {
                config: false,
                matches: true,
                packages: false,
            },
            ScopeSelection {
                config: false,
                matches: false,
                packages: true,
            },
            ScopeSelection::all(),
        ];

        for scope in scopes {
            let payload = export_to_vec(&paths, scope)?;

            let dest = TempDir::new("espanso-offline-dest")?;
            let dest_paths = create_sample_tree(dest.path())?;
            fs::write(dest_paths.config.join("config").join("default.yml"), "old")?;
            fs::write(dest_paths.config.join("match").join("base.yml"), "old")?;
            fs::write(dest_paths.packages.join("package.yml"), "old")?;

            import_from_bytes(&payload, &dest_paths, scope, false)?;

            if scope.config {
                assert_eq!(
                    fs::read_to_string(dest_paths.config.join("config").join("default.yml"))?,
                    "config"
                );
            } else {
                assert_eq!(
                    fs::read_to_string(dest_paths.config.join("config").join("default.yml"))?,
                    "old"
                );
            }

            if scope.matches {
                assert_eq!(
                    fs::read_to_string(dest_paths.config.join("match").join("base.yml"))?,
                    "matches"
                );
            } else {
                assert_eq!(
                    fs::read_to_string(dest_paths.config.join("match").join("base.yml"))?,
                    "old"
                );
            }

            if scope.packages {
                assert_eq!(
                    fs::read_to_string(dest_paths.packages.join("package.yml"))?,
                    "packages"
                );
            } else {
                assert_eq!(
                    fs::read_to_string(dest_paths.packages.join("package.yml"))?,
                    "old"
                );
            }
        }

        Ok(())
    }

    #[test]
    fn import_accepts_legacy_match_scope_prefix() -> Result<()> {
        let (scope, remainder) = split_scope_path(Path::new("match/base.yml"))?;
        assert_eq!(scope, Scope::Matches);
        assert_eq!(remainder, PathBuf::from("base.yml"));
        Ok(())
    }

    #[test]
    fn whitespace_filtering_reader_strips_whitespace() -> Result<()> {
        let input = b"a b\nc\t";
        let mut reader = WhitespaceFilteringReader::new(&input[..]);
        let mut output = String::new();
        reader.read_to_string(&mut output)?;
        assert_eq!(output, "abc");
        Ok(())
    }

    #[test]
    fn import_aborts_without_confirmation() -> Result<()> {
        let result = confirm_import_from_reader("n\n".as_bytes());
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn rejects_path_traversal() -> Result<()> {
        let mut raw = Vec::new();
        {
            let mut builder = Builder::new(&mut raw);
            let mut header = tar::Header::new_gnu();
            let name = b"matches/../evil";
            let old = header.as_old_mut();
            old.name = [0; 100];
            old.name[..name.len()].copy_from_slice(name);
            header.set_entry_type(EntryType::Regular);
            header.set_mode(0o644);
            header.set_size(4);
            header.set_cksum();
            builder.append(&header, "evil".as_bytes())?;
            builder.finish()?;
        }

        let mut gzipped = Vec::new();
        {
            let mut gzip = GzEncoder::new(&mut gzipped, Compression::default());
            gzip.write_all(&raw)?;
            gzip.finish()?;
        }

        let mut encoded = Vec::new();
        {
            let mut encoder = EncoderWriter::new(&mut encoded, &STANDARD);
            encoder.write_all(&gzipped)?;
            encoder.finish()?;
        }

        let temp = TempDir::new("espanso-offline")?;
        let paths = create_sample_tree(temp.path())?;
        let result = import_from_bytes(&encoded, &paths, ScopeSelection::all(), false);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn convert_line_breaks_for_yaml_only() -> Result<()> {
        let temp = TempDir::new("espanso-offline")?;
        let paths = create_sample_tree(temp.path())?;

        let yaml_path = paths.config.join("config").join("default.yml");
        let txt_path = paths.config.join("config").join("notes.txt");
        fs::write(&yaml_path, "line1\r\nline2\rline3")?;
        fs::write(&txt_path, "keep\r\nas-is")?;

        let payload = export_to_vec(
            &paths,
            ScopeSelection {
                config: true,
                matches: false,
                packages: false,
            },
        )?;

        let dest = TempDir::new("espanso-offline-dest")?;
        let dest_paths = create_sample_tree(dest.path())?;
        import_from_bytes(
            &payload,
            &dest_paths,
            ScopeSelection {
                config: true,
                matches: false,
                packages: false,
            },
            true,
        )?;

        let yaml_result = fs::read_to_string(dest_paths.config.join("config").join("default.yml"))?;
        let txt_result = fs::read_to_string(dest_paths.config.join("config").join("notes.txt"))?;
        assert_eq!(yaml_result, "line1\nline2\nline3");
        assert_eq!(txt_result, "keep\r\nas-is");

        Ok(())
    }
}
