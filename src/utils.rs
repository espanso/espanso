use std::path::Path;
use std::error::Error;
use walkdir::WalkDir;
use std::fs::create_dir;

pub fn copy_dir_into(source_dir: &Path, dest_dir: &Path) -> Result<(), Box<dyn Error>> {
    // Create source directory in dest dir
    let name = source_dir.file_name().expect("Error obtaining the filename");
    let target_dir = dest_dir.join(name);
    create_dir(&target_dir)?;

    for entry in WalkDir::new(source_dir) {
        let entry = entry?;
        let entry = entry.path();
        if entry.is_dir() {
            copy_dir_into(entry, &target_dir);
        }else if entry.is_file() {
            let target_entry = target_dir.join(entry.file_name().expect("Error obtaining the filename"));
            std::fs::copy(entry, target_entry);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::Path;
    use std::fs::create_dir;

    #[test]
    fn test_copy_dir_into() {
        let source_tmp_dir = TempDir::new().expect("Error creating temp directory");
        let dest_tmp_dir = TempDir::new().expect("Error creating temp directory");

        let source_dir = source_tmp_dir.path().join("source");
        create_dir(&source_dir);
        std::fs::write(source_dir.join("file1.txt"), "file1");
        std::fs::write(source_dir.join("file2.txt"), "file2");

        copy_dir_into(&source_dir, dest_tmp_dir.path());

        assert!(dest_tmp_dir.path().join("source").exists());
        assert!(dest_tmp_dir.path().join("source/file1.txt").exists());
        assert!(dest_tmp_dir.path().join("source/file2.txt").exists());
    }

    #[test]
    fn test_copy_dir_into_recursive() {
        let source_tmp_dir = TempDir::new().expect("Error creating temp directory");
        let dest_tmp_dir = TempDir::new().expect("Error creating temp directory");

        let source_dir = source_tmp_dir.path().join("source");
        create_dir(&source_dir);
        std::fs::write(source_dir.join("file1.txt"), "file1");
        std::fs::write(source_dir.join("file2.txt"), "file2");
        let nested_dir = source_dir.join("nested");
        create_dir(&nested_dir);
        std::fs::write(nested_dir.join("nestedfile.txt"), "nestedfile1");

        copy_dir_into(&source_dir, dest_tmp_dir.path());

        assert!(dest_tmp_dir.path().join("source").exists());
        assert!(dest_tmp_dir.path().join("source/file1.txt").exists());
        assert!(dest_tmp_dir.path().join("source/file2.txt").exists());

        assert!(dest_tmp_dir.path().join("source/nested").exists());
        assert!(dest_tmp_dir.path().join("source/nested/nestedfile.txt").exists());
    }

}