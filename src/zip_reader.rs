use memmap2::Mmap;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Cursor};
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use zip::ZipArchive;
use zip::read::ZipFile;

/// Extracts the contents of a ZIP archive into a target directory using memory-mapped I/O and parallelism.
///
/// # Arguments
/// - `zip_path`: Path to the ZIP file to be extracted.  
/// - `extract_path`: Path to the directory where files will be extracted.  
/// - `password`: Optional password used to decrypt encrypted entries. If `None`, only
///   unencrypted files will be accessible.  
///
/// # Behavior
/// - Uses [`memmap2`](https://docs.rs/memmap2/latest/memmap2/) to memory-map the entire ZIP file for efficient random access.  
/// - Uses [`rayon`](https://docs.rs/rayon/latest/rayon/) to extract files in parallel.  
/// - Attempts decryption with [`by_index_decrypt`](https://docs.rs/zip/latest/zip/read/struct.ZipArchive.html#method.by_index_decrypt) if `password` is provided.  
/// - Recreates directory structure as found in the ZIP archive.  
/// - Preserves relative paths; directory traversal protection (e.g., stripping `../`) should be added externally if required.  
///
/// # Performance
/// - Each parallel task re-initializes its own `ZipArchive` view over the shared memory-mapped file.  
/// - This avoids contention but increases overhead for archives with many entries.  
///
/// # Errors
/// Returns an error if:
/// - The ZIP file cannot be opened or memory-mapped.  
/// - The archive is corrupted or unreadable.  
/// - A file cannot be decrypted with the provided password.  
/// - Directories or files cannot be created under `extract_path`.  
/// - File write operations fail.  
///
/// # Security Notes
/// - Only legacy ZipCrypto is supported for decryption. This scheme is weak and may
///   incorrectly accept invalid passwords due to ZIP spec limitations.  
/// - AES-encrypted ZIPs may not be supported. Test with your archives before relying on this in production.  
///
/// # Panics
/// - Panics if a file entry in the ZIP archive does not have a valid parent directory path.  
///
/// # Example
/// ```rust,no_run
/// use zipoxide::extract_zip;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Extract unencrypted archive
///     extract_zip("archive.zip".to_string(), "output".to_string(), None)?;
///
///     // Extract password-protected archive
///     extract_zip("secret.zip".to_string(), "output".to_string(), Some("hunter2".to_string()))?;
///
///     Ok(())
/// }
/// ```

#[allow(unused)]
pub fn extract_zip(
    zip_path: String,
    extract_path: String,
    password: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let zip_path: &Path = Path::new(&zip_path);
    let extract_path: &Path = Path::new(&extract_path);
    let zip_file: File = File::open(zip_path)?;
    let mmap: Mmap = unsafe { Mmap::map(&zip_file)? }; // memory-map the whole zip
    let zip_archive: ZipArchive<Cursor<&[u8]>> = ZipArchive::new(std::io::Cursor::new(&mmap[..]))?;
    let indexes: Vec<usize> = (0..zip_archive.len()).collect();

    indexes.par_iter().try_for_each(
        |&index| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let mut zip_archive: ZipArchive<Cursor<&[u8]>> =
                ZipArchive::new(std::io::Cursor::new(&mmap[..]))?;
            let mut entry: ZipFile<'_, Cursor<&[u8]>> = match &password {
                Some(v) => zip_archive.by_index_decrypt(index, v.as_bytes())?,
                None => zip_archive.by_index(index)?,
            };
            let file_name: &str = entry.name();
            let output_path: PathBuf = extract_path.join(Path::new(file_name));
            if let Some(parent_dir) = output_path.parent() {
                if !parent_dir.exists() {
                    fs::create_dir_all(parent_dir)?;
                }
            }
            let mut file: File = File::create(output_path)?;
            io::copy(&mut entry, &mut file)?;
            Ok(())
        },
    );
    Ok(())
}

/// Reads all files inside a ZIP archive into memory buffers in parallel,
/// returning a `HashMap` where keys are file names and values are file contents.
///
/// # Arguments
/// - `zip_path`: Path to the ZIP file to be read.  
/// - `password`: Optional password used to decrypt encrypted files. If `None`, only
///   unencrypted entries will be accessible.  
///
/// # Behavior
/// - Uses [`memmap2`](https://docs.rs/memmap2/latest/memmap2/) to memory-map the entire ZIP file for efficient random access.  
/// - Uses [`rayon`](https://docs.rs/rayon/latest/rayon/) to read files in parallel.  
/// - If `password` is provided, attempts to decrypt each entry with [`by_index_decrypt`](https://docs.rs/zip/latest/zip/read/struct.ZipArchive.html#method.by_index_decrypt).  
/// - Stores each file's full contents into a `Vec<u8>` in memory.  
/// - File names are taken directly from the ZIP archive’s metadata (UTF-8 required).  
///
/// # Concurrency Model
/// - Each worker thread creates its own `ZipArchive` view over the shared memory-mapped file.  
/// - Results are collected in a thread-safe `Arc<Mutex<HashMap<...>>>` before being unwrapped.  
/// - Suitable for archives with many medium-sized files; overhead may dominate if there are only a few entries.  
///
/// # Returns
/// A `HashMap<String, Vec<u8>>` where:
/// - Key = file name inside the ZIP archive.  
/// - Value = full file contents as bytes.  
///
/// # Errors
/// Returns an error if:
/// - The ZIP file cannot be opened or memory-mapped.  
/// - The archive is corrupted or unreadable.  
/// - A file cannot be extracted or decrypted (wrong password).  
///
/// # Panics
/// - Panics if the `Arc<Mutex<_>>` cannot be unwrapped (only occurs if still shared, which should not happen here).  
///
/// # Security Notes
/// - Only legacy ZipCrypto is supported for decryption. This scheme is weak and may
///   incorrectly accept invalid passwords due to ZIP spec limitations.  
/// - AES-encrypted ZIP files may not be supported. Test with your target archives.  
///
/// # Example
/// ```rust,no_run
/// use zipoxide::read_zip_contents_into_buffer;
///
/// fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     // Unencrypted ZIP
///     let plain = read_zip_contents_into_buffer("plain.zip".to_string(), None)?;
///     println!("Read {} files", plain.len());
///
///     // Encrypted ZIP
///     let secret = read_zip_contents_into_buffer("secret.zip".to_string(), Some("hunter2".to_string()))?;
///     for (name, data) in secret {
///         println!("File: {}, Size: {} bytes", name, data.len());
///     }
///     Ok(())
/// }
/// ```
#[allow(unused)]
pub fn read_zip_contents_into_buffer(
    zip_path: String,
    password: Option<String>,
) -> Result<HashMap<String, Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
    let zip_path: &Path = Path::new(&zip_path);
    let file: File = File::open(zip_path)?;
    let mmap: Mmap = unsafe { Mmap::map(&file)? }; // memory-map the whole zip

    let zip_archive: ZipArchive<Cursor<&[u8]>> = ZipArchive::new(std::io::Cursor::new(&mmap[..]))?;
    let shared_results: Arc<Mutex<HashMap<String, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));

    let indexes: Vec<usize> = (0..zip_archive.len()).collect();

    indexes.par_iter().try_for_each(
        |&index| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let mut zip_archive: ZipArchive<Cursor<&[u8]>> =
                ZipArchive::new(std::io::Cursor::new(&mmap[..]))?;
            let mut entry: ZipFile<'_, Cursor<&[u8]>> = match &password {
                Some(v) => zip_archive.by_index_decrypt(index, v.as_bytes())?,
                None => zip_archive.by_index(index)?,
            };

            let file_name: String = entry.name().to_string();
            let mut buffer = Vec::with_capacity(entry.size() as usize);
            io::copy(&mut entry, &mut buffer)?;

            shared_results.lock().unwrap().insert(file_name, buffer);

            Ok(())
        },
    )?;

    Ok(Arc::try_unwrap(shared_results).unwrap().into_inner()?)
}
