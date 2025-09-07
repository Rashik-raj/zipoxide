# zipoxide

A Rust-powered, blazing fast ZIP utility.
Leverages memory-mapped I/O and parallelism to read, create, and extract ZIP archives efficiently.

---

## ⚡ Features

* **Read ZIP contents in parallel** into memory buffers (`HashMap<String, Vec<u8>>`), including password-protected archives.
* **Create ZIP archives** from folders or lists of files, preserving directory structure, with configurable options.
* **Extract ZIP archives** in parallel using memory-mapped I/O, supporting optional passwords.
* Pure Rust, high-performance ZIP operations.

---

## 📦 Installation

Add `zipoxide` to your `Cargo.toml`:

```toml
[dependencies]
zipoxide = "0.1"
```

Then build your project:

```bash
cargo build
```

---

## 🧩 Usage

### Read ZIP Contents into Memory

```rust,no_run
use zipoxide::read_zip_contents_into_buffer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Unencrypted ZIP
    let contents = read_zip_contents_into_buffer("archive.zip".to_string(), None)?;
    
    // Password-protected ZIP
    let protected_contents = read_zip_contents_into_buffer(
        "secret.zip".to_string(),
        Some("hunter2".to_string()),
    )?;

    for (name, data) in protected_contents {
        println!("File: {}, Size: {} bytes", name, data.len());
    }
    Ok(())
}
```

* **Returns:** `HashMap<String, Vec<u8>>` where key = file name, value = file bytes.
* **Use case:** Quickly access all files in a ZIP archive in memory, including encrypted files.

---

### Create ZIP from a Folder

```rust,no_run
use zipoxide::create_zip_from_folder;
use zip::write::FileOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = FileOptions::default();
    create_zip_from_folder(
        "archive.zip".to_string(),
        "my_folder".to_string(),
        options,
    )?;
    Ok(())
}
```

* Recursively compresses folder contents.
* Preserves directory structure inside the archive.
* Supports optional encryption via `FileOptions::encrypt_with(password)`.

---

### Create ZIP from Files/Directories

```rust,no_run
use zipoxide::create_zip_from_files;
use zip::write::FileOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = FileOptions::default();
    create_zip_from_files(
        "files_archive.zip".to_string(),
        vec!["file1.txt".to_string(), "dir1".to_string()],
        options,
    )?;
    Ok(())
}
```

* Accepts a list of files or directories.
* Directories are recursively compressed.
* Paths inside the archive are relative to the input paths.
* Supports password-protected archives via `FileOptions`.

---

### Extract ZIP Archive

```rust,no_run
use zipoxide::extract_zip;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Unencrypted extraction
    extract_zip(
        "archive.zip".to_string(),
        "output_dir".to_string(),
        None,
    )?;

    // Password-protected extraction
    extract_zip(
        "secret.zip".to_string(),
        "output_dir_secure".to_string(),
        Some("hunter2".to_string()),
    )?;

    Ok(())
}
```

* Recreates the directory structure from the archive.
* Uses memory-mapped I/O and parallelism for faster extraction.
* Supports optional password for encrypted archives.

---

## 🏗 Architecture Notes

* **Memory-mapped I/O:** Efficient random access for reading and extracting files.
* **Parallelism:** Uses [Rayon](https://docs.rs/rayon/latest/rayon/) for parallelism.
* **Thread safety:** `Arc<Mutex<...>>` ensures safe parallel writes to in-memory structures.

---

## ⚠️ Limitations

* Overhead of parallelism may outweigh benefits for archives with very few files.
* Panics if output ZIP file already exists.

---

## 📈 Performance

* Parallel reading and extraction scale with CPU cores.
* Memory-mapped I/O reduces disk read overhead.
* Optimized for large archives with many files.

---

## 🔧 Dependencies

* [zip](https://crates.io/crates/zip) – ZIP reading/writing.
* [memmap2](https://crates.io/crates/memmap2) – Memory-mapped I/O.
* [rayon](https://crates.io/crates/rayon) – Parallelism.

---

## 📝 License

MIT License © \[Rashikraj Shrestha]
