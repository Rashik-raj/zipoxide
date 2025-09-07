use std::io;
use std::fs::{self, File, DirEntry};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::ZipWriter;

/// Creates a ZIP archive from the contents of a folder, including all nested files and subdirectories.
///
/// # Arguments
/// - `output_zip_path`: Path where the resulting ZIP archive will be created.  
/// - `folder_path`: Root folder whose contents (including subdirectories) will be compressed into the ZIP.  
/// - `zip_options`: [`zip::write::FileOptions`] specifying compression method, permissions, etc.
///
/// # Behavior
/// - Preserves the relative directory structure inside the archive.  
/// - Recursively traverses subdirectories.  
/// - Panics if the output ZIP file already exists.  
/// - Non-UTF8 file paths will cause a runtime error.  
///
/// # Errors
/// Returns an error if:
/// - The folder path does not exist or cannot be read.  
/// - A file cannot be opened or read.  
/// - Writing to the ZIP archive fails.  
///
/// # Example
/// ```rust,no_run
/// use zipoxide::create_zip_from_folder;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     create_zip_from_folder(
///         "archive.zip".to_string(),
///         "my_folder".to_string(),
///         zip::write::FileOptions::default(),
///     )?;
///     Ok(())
/// }
/// ```
#[allow(unused)]
pub fn create_zip_from_folder(
    output_zip_path: String,
    folder_path: String,
    zip_options: FileOptions<'static, ()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_zip_path: &Path = Path::new(&output_zip_path);
    if output_zip_path.exists() {
        panic!("Output zip path already exists.");
    }
    let folder_path: &Path = Path::new(&folder_path);

    let zip_file: File = File::create(output_zip_path)?;
    let mut zip_writer: ZipWriter<File> = ZipWriter::new(zip_file);

    let mut directories_to_visit: Vec<PathBuf> = vec![folder_path.to_path_buf()];

    while let Some(current_dir) = directories_to_visit.pop() {
        for entry in fs::read_dir(&current_dir)? {
            let entry: DirEntry = entry?;
            let path: PathBuf = entry.path();
            let relative_path: &Path = path.strip_prefix(folder_path)?;

            if path.is_dir() {
                directories_to_visit.push(path);
            } else {
                zip_writer.start_file(relative_path.to_str().unwrap(), zip_options)?;
                let mut f: File = File::open(&path)?;
                io::copy(&mut f, &mut zip_writer)?;
            }
        }
    }

    zip_writer.finish()?;
    Ok(())
}

/// Creates a ZIP archive from a list of files and/or directories.
///
/// # Arguments
/// - `output_zip_path`: Path where the resulting ZIP archive will be created.  
/// - `files_path`: List of file or directory paths to include in the ZIP archive.  
///   - Directories are added recursively.  
///   - Relative paths are preserved.  
/// - `zip_options`: [`zip::write::FileOptions`] specifying compression method, permissions, etc.
///
/// # Behavior
/// - Handles both files and directories.  
/// - Panics if the output ZIP file already exists.  
/// - Non-UTF8 file paths will return an error.  
///
/// # Errors
/// Returns an error if:
/// - Any input path does not exist or is invalid.  
/// - A file cannot be opened or read.  
/// - Writing to the ZIP archive fails.  
///
/// # Example
/// ```rust,no_run
/// use zipoxide::create_zip_from_files;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     create_zip_from_files(
///         "files_archive.zip".to_string(),
///         vec!["file1.txt".to_string(), "dir1".to_string()],
///         zip::write::FileOptions::default(),
///     )?;
///     Ok(())
/// }
/// ```
#[allow(unused)]
pub fn create_zip_from_files(
    output_zip_path: String,
    files_path: Vec<String>,
    zip_options: FileOptions<'static, ()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_zip_path: &Path = Path::new(&output_zip_path);
    if output_zip_path.exists() {
        panic!("Output zip path already exists.");
    }
    let zip_file: File = File::create(output_zip_path)?;
    let mut zip_writer: ZipWriter<File> = ZipWriter::new(zip_file);

    let mut stack: Vec<(PathBuf, PathBuf)> = Vec::new();

    for file_path_str in files_path.iter() {
        let path: PathBuf = PathBuf::from(file_path_str);
        let relative_path: std::ffi::OsString = path.file_name().ok_or("Invalid file name")?.to_owned();
        stack.push((path, PathBuf::from(relative_path)));
    }

    while let Some((full_path, relative_path)) = stack.pop() {
        if full_path.is_dir() {
            for entry in fs::read_dir(&full_path)? {
                let entry: DirEntry = entry?;
                let entry_path: PathBuf = entry.path();
                let entry_relative_path: PathBuf = relative_path.join(entry.file_name());
                stack.push((entry_path, entry_relative_path));
            }
        } else if full_path.is_file() {
            zip_writer.start_file(relative_path.to_str().unwrap(), zip_options)?;
            let mut file: File = File::open(&full_path)?;
            io::copy(&mut file, &mut zip_writer)?;
        }
    }

    zip_writer.finish()?;
    Ok(())
}
