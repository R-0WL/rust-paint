use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Reserve a new numbered folder after the largest existing number.
/// create_dir is atomic: another running instance cannot claim the same folder.
pub fn reserve_export_directory(base: &Path) -> io::Result<PathBuf> {
    let mut largest = 0u64;
    for entry in fs::read_dir(base)? {
        let entry = entry?;
        if let Some(number) = entry.file_name().to_str().and_then(|s| s.parse::<u64>().ok()) {
            largest = largest.max(number);
        }
    }
    loop {
        largest = largest.checked_add(1)
            .ok_or_else(|| io::Error::other("Se agotó la numeración de exportaciones"))?;
        let directory = base.join(largest.to_string());
        match fs::create_dir(&directory) {
            Ok(()) => return Ok(directory),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_continue_after_restart_and_preserve_existing_files() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let base = std::env::temp_dir().join(format!("paint-exports-{}-{unique}", std::process::id()));
        fs::create_dir(&base).unwrap();
        let result = (|| -> io::Result<()> {
            assert_eq!(reserve_export_directory(&base)?, base.join("1"));
            fs::write(base.join("1/owl.bmp"), b"previous drawing")?;
            assert_eq!(reserve_export_directory(&base)?, base.join("2"));
            // Existing higher numbers are respected, even when there are gaps.
            fs::create_dir(base.join("8"))?;
            assert_eq!(reserve_export_directory(&base)?, base.join("9"));
            // A numeric filename also reserves its number.
            fs::write(base.join("10"), b"keep this file")?;
            assert_eq!(reserve_export_directory(&base)?, base.join("11"));
            assert_eq!(fs::read(base.join("1/owl.bmp"))?, b"previous drawing");
            assert_eq!(fs::read(base.join("10"))?, b"keep this file");
            Ok(())
        })();
        fs::remove_dir_all(&base).unwrap();
        result.unwrap();
    }
}
