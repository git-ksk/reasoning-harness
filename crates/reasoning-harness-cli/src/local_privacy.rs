use std::{fs, io::Write, path::Path};

#[cfg(unix)]
fn check_owner(path: &Path, metadata: &fs::Metadata) -> Result<(), String> {
    use std::os::unix::fs::MetadataExt;
    let owner = metadata.uid();
    let current = unsafe { libc::geteuid() };
    if owner != current {
        return Err(format!(
            "{} is owned by uid {owner}, not the current uid {current}; refusing private Reason state",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_owner(_path: &Path, _metadata: &fs::Metadata) -> Result<(), String> {
    Ok(())
}

pub(crate) fn ensure_private_directory(path: &Path) -> Result<(), String> {
    if path.exists() {
        let metadata = fs::metadata(path).map_err(|error| {
            format!(
                "{}: cannot inspect private directory: {error}",
                path.display()
            )
        })?;
        check_owner(path, &metadata)?;
        if !metadata.is_dir() {
            return Err(format!(
                "{}: private state path is not a directory",
                path.display()
            ));
        }
    } else {
        fs::create_dir_all(path).map_err(|error| {
            format!(
                "{}: cannot create private directory: {error}",
                path.display()
            )
        })?;
        let metadata = fs::metadata(path).map_err(|error| {
            format!(
                "{}: cannot inspect private directory: {error}",
                path.display()
            )
        })?;
        check_owner(path, &metadata)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|error| {
            format!(
                "{}: cannot set private directory permissions: {error}",
                path.display()
            )
        })?;
        let mode = fs::metadata(path)
            .map_err(|error| {
                format!(
                    "{}: cannot inspect private directory: {error}",
                    path.display()
                )
            })?
            .permissions()
            .mode();
        if mode & 0o077 != 0 {
            return Err(format!(
                "{}: Reason private directory remains group/world accessible",
                path.display()
            ));
        }
    }
    Ok(())
}

pub(crate) fn ensure_private_file(path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("{}: cannot inspect private file: {error}", path.display()))?;
    check_owner(path, &metadata)?;
    if !metadata.is_file() {
        return Err(format!(
            "{}: private state path is not a regular file",
            path.display()
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|error| {
            format!(
                "{}: cannot set private file permissions: {error}",
                path.display()
            )
        })?;
        let mode = fs::metadata(path)
            .map_err(|error| format!("{}: cannot inspect private file: {error}", path.display()))?
            .permissions()
            .mode();
        if mode & 0o077 != 0 {
            return Err(format!(
                "{}: Reason private file remains group/world readable/writable",
                path.display()
            ));
        }
    }
    Ok(())
}

pub(crate) fn write_new_private_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|error| {
                format!("{}: cannot create export parent: {error}", parent.display())
            })?;
        }
    }
    #[cfg(unix)]
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(path)
    }
    .map_err(|error| format!("{}: cannot create private file: {error}", path.display()))?;
    #[cfg(not(unix))]
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| format!("{}: cannot create private file: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("{}: cannot write private file: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("{}: cannot sync private file: {error}", path.display()))?;
    ensure_private_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(unix)]
    #[test]
    fn fresh_private_directory_and_file_are_user_only() {
        use std::os::unix::fs::PermissionsExt;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("reason-private-{}-{stamp}", std::process::id()));
        ensure_private_directory(&root).unwrap();
        let file = root.join("state.json");
        fs::write(&file, b"state").unwrap();
        ensure_private_file(&file).unwrap();
        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&file).unwrap().permissions().mode() & 0o777,
            0o600
        );
        fs::remove_dir_all(root).unwrap();
    }
}
