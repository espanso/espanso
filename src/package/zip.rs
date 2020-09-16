use log::debug;
use std::error::Error;
use std::io::{copy, Cursor};
use std::{fs, io};
use tempfile::TempDir;

pub struct ZipPackageResolver;

impl ZipPackageResolver {
    pub fn new() -> ZipPackageResolver {
        return ZipPackageResolver {};
    }
}

impl super::PackageResolver for ZipPackageResolver {
    fn clone_repo_to_temp(&self, repo_url: &str, proxy: Option<String>) -> Result<TempDir, Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        let zip_url = repo_url.to_owned() + "/archive/master.zip";

        let mut client = reqwest::Client::builder();

        if let Some(proxy) = proxy {
            let proxy = reqwest::Proxy::https(&proxy).expect("unable to setup https proxy");
            client = client.proxy(proxy);
        };

        let client = client.build().expect("unable to create http client");

        // Download the archive from GitHub
        let mut response = client.get(&zip_url).send()?;

        // Extract zip file
        let mut buffer = Vec::new();
        copy(&mut response, &mut buffer)?;

        let reader = Cursor::new(buffer);

        let mut archive = zip::ZipArchive::new(reader).unwrap();

        // Find the root folder name
        let mut root_folder = {
            let root_folder = archive.by_index(0).unwrap();
            let root_folder = root_folder.sanitized_name();
            root_folder.to_str().unwrap().to_owned()
        };
        root_folder.push(std::path::MAIN_SEPARATOR);

        for i in 1..archive.len() {
            let mut file = archive.by_index(i).unwrap();

            let current_path = file.sanitized_name();
            let current_filename = current_path.to_str().unwrap();
            let trimmed_filename = current_filename.trim_start_matches(&root_folder);

            let outpath = temp_dir.path().join(trimmed_filename);

            {
                let comment = file.comment();
                if !comment.is_empty() {
                    debug!("File {} comment: {}", i, comment);
                }
            }

            if (&*file.name()).ends_with('/') {
                debug!(
                    "File {} extracted to \"{}\"",
                    i,
                    outpath.as_path().display()
                );
                fs::create_dir_all(&outpath).unwrap();
            } else {
                debug!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.as_path().display(),
                    file.size()
                );
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }
        }

        Ok(temp_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::super::PackageResolver;
    use super::*;

    #[test]
    fn test_clone_temp_repository() {
        let resolver = ZipPackageResolver::new();
        let cloned_dir = resolver
            .clone_repo_to_temp("https://github.com/federico-terzi/espanso-hub-core", None)
            .unwrap();
        assert!(cloned_dir.path().join("LICENSE").exists());
    }
}
