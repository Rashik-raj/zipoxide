#[cfg(test)]
mod tests {
    use zipoxide::{
        read_zip_contents_into_buffer, create_zip_from_folder, create_zip_from_files, extract_zip,
    };
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use zip::write::FileOptions;

    fn default_options() -> FileOptions<'static, ()> {
        FileOptions::default()
    }

    #[test]
    fn test_read_zip_contents_into_buffer() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test.zip");
        let file_path = dir.path().join("file.txt");

        // Create a test file
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "Hello ZIP!").unwrap();

        // Create a ZIP archive
        create_zip_from_files(
            zip_path.to_str().unwrap().to_string(),
            vec![file_path.to_str().unwrap().to_string()],
            default_options(),
        )
        .unwrap();

        // Read ZIP contents
        let contents = read_zip_contents_into_buffer(zip_path.to_str().unwrap().to_string(), None).unwrap();
        assert_eq!(contents.len(), 1);
        assert!(contents.contains_key("file.txt"));
        assert_eq!(contents["file.txt"], b"Hello ZIP!\n");
    }

    #[test]
    fn test_create_zip_from_folder() {
        let dir = tempdir().unwrap();
        let folder = dir.path().join("my_folder");
        fs::create_dir(&folder).unwrap();

        let file1 = folder.join("a.txt");
        let file2 = folder.join("b.txt");
        fs::write(&file1, b"Hello").unwrap();
        fs::write(&file2, b"World").unwrap();

        let zip_path = dir.path().join("folder.zip");
        create_zip_from_folder(
            zip_path.to_str().unwrap().to_string(),
            folder.to_str().unwrap().to_string(),
            default_options(),
        )
        .unwrap();

        let contents = read_zip_contents_into_buffer(zip_path.to_str().unwrap().to_string(), None).unwrap();
        assert_eq!(contents.len(), 2);
        assert_eq!(contents["a.txt"], b"Hello");
        assert_eq!(contents["b.txt"], b"World");
    }

    #[test]
    fn test_create_zip_from_files() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("x.txt");
        let file2 = dir.path().join("y.txt");
        fs::write(&file1, b"Foo").unwrap();
        fs::write(&file2, b"Bar").unwrap();

        let zip_path = dir.path().join("files.zip");
        create_zip_from_files(
            zip_path.to_str().unwrap().to_string(),
            vec![file1.to_str().unwrap().to_string(), file2.to_str().unwrap().to_string()],
            default_options(),
        )
        .unwrap();

        let contents = read_zip_contents_into_buffer(zip_path.to_str().unwrap().to_string(), None).unwrap();
        assert_eq!(contents.len(), 2);
        assert_eq!(contents["x.txt"], b"Foo");
        assert_eq!(contents["y.txt"], b"Bar");
    }

    #[test]
    fn test_extract_zip() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("file.txt");
        fs::write(&file, b"Data").unwrap();

        let zip_path = dir.path().join("archive.zip");
        create_zip_from_files(
            zip_path.to_str().unwrap().to_string(),
            vec![file.to_str().unwrap().to_string()],
            default_options(),
        )
        .unwrap();

        let extract_dir = dir.path().join("extract");
        extract_zip(
            zip_path.to_str().unwrap().to_string(),
            extract_dir.to_str().unwrap().to_string(),
            None,
        )
        .unwrap();

        let extracted_file = extract_dir.join("file.txt");
        assert!(extracted_file.exists());
        let content = fs::read(&extracted_file).unwrap();
        assert_eq!(content, b"Data");
    }

    #[test]
    fn test_extract_zip_preserves_structure() {
        let dir = tempdir().unwrap();
        let folder = dir.path().join("nested");
        fs::create_dir(&folder).unwrap();
        fs::write(folder.join("inner.txt"), b"Nested content").unwrap();

        let zip_path = dir.path().join("nested.zip");
        create_zip_from_folder(
            zip_path.to_str().unwrap().to_string(),
            folder.to_str().unwrap().to_string(),
            default_options(),
        )
        .unwrap();

        let extract_dir = dir.path().join("extract_nested");
        extract_zip(
            zip_path.to_str().unwrap().to_string(),
            extract_dir.to_str().unwrap().to_string(),
            None,
        )
        .unwrap();

        let extracted_file = extract_dir.join("inner.txt");
        assert!(extracted_file.exists());
        let content = fs::read(&extracted_file).unwrap();
        assert_eq!(content, b"Nested content");
    }

    #[test]
    fn test_create_zip_from_files_with_subdir() {
        // Verify that directory inside files list is recursively zipped
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("sub");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("subfile.txt"), b"Subfile").unwrap();

        let file = dir.path().join("root.txt");
        fs::write(&file, b"Root").unwrap();

        let zip_path = dir.path().join("combined.zip");
        create_zip_from_files(
            zip_path.to_str().unwrap().to_string(),
            vec![file.to_str().unwrap().to_string(), subdir.to_str().unwrap().to_string()],
            default_options(),
        )
        .unwrap();

        let contents = read_zip_contents_into_buffer(zip_path.to_str().unwrap().to_string(), None).unwrap();
        assert_eq!(contents.len(), 2);
        assert_eq!(contents["root.txt"], b"Root");
        assert_eq!(contents["sub/subfile.txt"], b"Subfile");
    }

    #[test]
    #[should_panic(expected = "Output zip path already exists.")]
    fn test_create_zip_from_folder_panics_on_existing_zip() {
        let dir = tempdir().unwrap();
        let folder = dir.path().join("folder");
        fs::create_dir(&folder).unwrap();
        let zip_path = dir.path().join("exists.zip");
        fs::File::create(&zip_path).unwrap();

        create_zip_from_folder(
            zip_path.to_str().unwrap().to_string(),
            folder.to_str().unwrap().to_string(),
            default_options(),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Output zip path already exists.")]
    fn test_create_zip_from_files_panics_on_existing_zip() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("file.txt");
        fs::write(&file, b"data").unwrap();
        let zip_path = dir.path().join("exists.zip");
        fs::File::create(&zip_path).unwrap();

        create_zip_from_files(
            zip_path.to_str().unwrap().to_string(),
            vec![file.to_str().unwrap().to_string()],
            default_options(),
        )
        .unwrap();
    }

    #[test]
    fn test_read_and_extract_password_protected_zip() {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let zip_path = project_root.join("tests/protected.zip");
        let password = "test";

        // 1️⃣ Read the contents into memory
        let contents = read_zip_contents_into_buffer(zip_path.to_str().unwrap().to_string(), Some(password.to_string()))
            .expect("Failed to read password-protected ZIP");

        assert_eq!(contents.len(), 1);
        assert!(contents.contains_key("test.txt"));
        assert_eq!(contents["test.txt"], b"This is password protection read test.");

        // 2️⃣ Extract the ZIP to a temporary directory
        let temp_dir = tempdir().unwrap();
        let extract_path = temp_dir.path().to_str().unwrap();

        extract_zip(zip_path.to_str().unwrap().to_string(), extract_path.to_string(), Some(password.to_string()))
            .expect("Failed to extract password-protected ZIP");

        // Verify extracted file
        let extracted_file_path = temp_dir.path().join("test.txt");
        assert!(extracted_file_path.exists(), "Extracted file does not exist");

        let content = fs::read(&extracted_file_path).expect("Failed to read extracted file");
        assert_eq!(content, b"This is password protection read test.");
    }

    
}
