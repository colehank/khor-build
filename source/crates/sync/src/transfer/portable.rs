//! Directory publication on Unix filesystems without exclusive rename.
//!
//! The three controls keep their original encoding. An atomic, create-only
//! symlink publishes a fully initialized object in a reserved private namespace.
//! Only the exact relative target is accepted; arbitrary symlinks stay errors.
//! Older readers refuse this namespace. Once its last stage is cancelled it is
//! removed, restoring the old layout. No production root is moved or migrated.

use super::*;

pub(super) const DIRECTORY: &str = ".stages";

pub(super) fn unsupported(why: &io::Error) -> bool {
    cfg!(unix)
        && (why.kind() == io::ErrorKind::Unsupported
            || (cfg!(any(target_os = "linux", target_os = "android"))
                && matches!(why.raw_os_error(), Some(22 | 38 | 95))))
}

fn id_name(name: &str) -> bool {
    name.len() == 64
        && name
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn owned_dir(root: &Path, path: &Path) -> Result<()> {
    check_dir(path)?;
    check_initialization_private(&fs::symlink_metadata(path)?, &fs::symlink_metadata(root)?)
}

pub(super) fn check_container(root: &Path) -> Result<()> {
    if !cfg!(unix) {
        return Err(Error::InvalidStage(
            "portable stages require Unix link semantics",
        ));
    }
    owned_dir(root, &root.join(DIRECTORY))
}

fn target(name: &str) -> PathBuf {
    Path::new(DIRECTORY).join(name)
}

fn published(root: &Path, name: &str) -> Result<bool> {
    let path = root.join(name);
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(why) if why.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(why) => return Err(why.into()),
    };
    if !meta.file_type().is_symlink() {
        return Ok(false);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.uid() != fs::symlink_metadata(root)?.uid() {
            return Err(Error::InvalidStage(
                "published link owner differs from root",
            ));
        }
    }
    Ok(fs::read_link(path)? == target(name))
}

pub(super) fn resolve(root: &Path, name: &str) -> Result<PathBuf> {
    let path = root.join(name);
    if !id_name(name) {
        return Err(Error::InvalidStage("invalid portable stage id"));
    }
    if fs::symlink_metadata(&path)?.file_type().is_symlink() {
        if !published(root, name)? {
            return Err(Error::InvalidStage("unexpected published link target"));
        }
        check_container(root)?;
        let object = root.join(target(name));
        owned_dir(root, &object)?;
        Ok(object)
    } else {
        check_dir(&path)?;
        Ok(path)
    }
}

// Validate every entry before unlinking any. Unpublished objects may contain
// only the same bounded, possibly partial controls as the original initializer.
fn controls(root: &Path, dir: &Path, initializing: bool) -> Result<Vec<PathBuf>> {
    owned_dir(root, dir)?;
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)?.take(4) {
        let entry = entry?;
        let limit = match entry.file_name().to_str() {
            Some("manifest") => MAX_METADATA_BYTES as u64,
            Some("body") => {
                if initializing {
                    0
                } else {
                    u64::MAX
                }
            }
            Some("offsets") => {
                if initializing {
                    RECORD_BYTES as u64
                } else {
                    u64::MAX
                }
            }
            _ => return Err(Error::InvalidStage("unknown portable stage control")),
        };
        let path = entry.path();
        check_file(&path)?;
        let file = File::open(&path)?;
        let held = file.metadata()?;
        check_initialization_private(&held, &fs::symlink_metadata(root)?)?;
        if held.len() > limit {
            return Err(Error::InvalidStage("portable control exceeds its bound"));
        }
        if initializing || entry.file_name() != "manifest" {
            platform::check_control_file(&file)?;
        }
        files.push(path);
    }
    Ok(files)
}

fn remove_controls(dir: &Path, files: Vec<PathBuf>) -> Result<()> {
    for path in files {
        fs::remove_file(path)?;
    }
    fs::remove_dir(dir)?;
    Ok(())
}

fn remove_empty_container(root: &Path) -> Result<()> {
    let dir = root.join(DIRECTORY);
    if fs::read_dir(&dir)?.next().is_none() {
        fs::remove_dir(dir)?;
    }
    Ok(())
}

pub(super) fn publish(root: &Path, manifest: &Manifest, id: TransferId) -> Result<()> {
    #[cfg(unix)]
    {
        let parent = root.join(DIRECTORY);
        private_dir(&parent)?;
        check_container(root)?;
        let name = id.to_string();
        // Reject an already dangling, but otherwise well-shaped link before
        // creating its target. Atomic symlink creation still handles races.
        match fs::symlink_metadata(root.join(&name)) {
            Ok(_) => return Err(io::Error::from(io::ErrorKind::AlreadyExists).into()),
            Err(why) if why.kind() == io::ErrorKind::NotFound => {}
            Err(why) => return Err(why.into()),
        }
        let object = parent.join(&name);
        create_dir(&object)?;
        #[cfg(test)]
        super::tests::pause_after_stage_mkdir(&object);
        initialize(&object, manifest, id)?;
        if read_manifest(&object)? != *manifest
            || fs::read(object.join("offsets"))? != checkpoint(id, 0, *blake3::hash(&[]).as_bytes())
        {
            return Err(Error::InvalidStage("portable initializer binding changed"));
        }
        controls(root, &object, true)?;
        #[cfg(test)]
        super::tests::pause_initialization(&object, "portable_before_publish");
        let result = match std::os::unix::fs::symlink(target(&name), root.join(&name)) {
            Ok(()) => Ok(()),
            // NFS may have committed an operation whose reply was lost. The
            // immutable manifest and exact target must prove that publication.
            Err(why)
                if why.kind() != io::ErrorKind::AlreadyExists
                    && published(root, &name)?
                    && read_manifest(&object)? == *manifest =>
            {
                Ok(())
            }
            Err(why) => Err(why.into()),
        };
        #[cfg(test)]
        if result.is_ok() {
            super::tests::pause_initialization(&object, "portable_after_publish");
        }
        result
    }
    #[cfg(not(unix))]
    {
        let _ = (root, manifest, id);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "portable publication needs Unix symlinks",
        )
        .into())
    }
}

fn marker(root: &Path, name: &str) -> PathBuf {
    root.join(DIRECTORY).join(format!("{name}.cancel"))
}

fn cancellation(root: &Path, name: &str) -> Result<Manifest> {
    let path = marker(root, name);
    check_file(&path)?;
    let mut file = File::open(&path)?;
    check_initialization_private(&file.metadata()?, &fs::symlink_metadata(root)?)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take((MAX_METADATA_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    let manifest = Manifest::decode(&bytes)?;
    if manifest.id()?.to_string() != name {
        return Err(Error::InvalidStage("cancel marker does not bind stage"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let held = file.metadata()?;
        let meta = root.join(target(name)).join("manifest");
        match fs::symlink_metadata(meta) {
            Ok(original)
                if !is_link(&original)
                    && original.is_file()
                    && original.dev() == held.dev()
                    && original.ino() == held.ino()
                    && held.nlink() == 2 => {}
            Err(why) if why.kind() == io::ErrorKind::NotFound && held.nlink() == 1 => {}
            _ => return Err(Error::InvalidStage("cancel marker link ownership changed")),
        }
    }
    Ok(manifest)
}

fn finish_cancel(root: &Path, name: &str) -> Result<()> {
    cancellation(root, name)?;
    let object = root.join(target(name));
    let files = match fs::symlink_metadata(&object) {
        Ok(_) => Some(controls(root, &object, false)?),
        Err(why) if why.kind() == io::ErrorKind::NotFound => None,
        Err(why) => return Err(why.into()),
    };
    #[cfg(test)]
    super::tests::pause_initialization(&object, "portable_cancel_marked");
    match fs::symlink_metadata(root.join(name)) {
        Ok(_) if published(root, name)? => fs::remove_file(root.join(name))?,
        Ok(_) => return Err(Error::InvalidStage("cancel target is not the owned link")),
        Err(why) if why.kind() == io::ErrorKind::NotFound => {}
        Err(why) => return Err(why.into()),
    }
    #[cfg(test)]
    super::tests::pause_initialization(&object, "portable_cancel_unlinked");
    if let Some(files) = files {
        for path in files {
            fs::remove_file(path)?;
            #[cfg(test)]
            super::tests::pause_initialization(&object, "portable_cancel_control_removed");
        }
        fs::remove_dir(&object)?;
    }
    #[cfg(test)]
    super::tests::pause_initialization(&object, "portable_cancel_object_removed");
    fs::remove_file(marker(root, name))?;
    #[cfg(test)]
    super::tests::pause_initialization(&object, "portable_cancel_marker_removed");
    Ok(())
}

pub(super) fn cancel(
    root: &Path,
    expected: &Manifest,
    id: TransferId,
) -> Result<Option<Cancelled>> {
    let name = id.to_string();
    if !published(root, &name)? {
        return Ok(None);
    }
    let object = resolve(root, &name)?;
    if read_manifest(&object)? != *expected {
        return Err(Error::InvalidStage("cancel manifest changed"));
    }
    controls(root, &object, false)?;
    platform::check_control_file(&File::open(object.join("manifest"))?)?;
    // An atomic hard link is a complete cancellation intent, including after
    // a crash before the first public unlink. A partial marker is impossible.
    match fs::hard_link(object.join("manifest"), marker(root, &name)) {
        Ok(()) => {}
        Err(why) => {
            if cancellation(root, &name).ok().as_ref() != Some(expected) {
                return Err(why.into());
            }
        }
    }
    finish_cancel(root, &name)?;
    remove_empty_container(root)?;
    Ok(Some(Cancelled::Removed))
}

pub(super) fn reconcile(store: &TransferStore, held: &Gate<'_>) -> Result<()> {
    let root = &store.root;
    match fs::symlink_metadata(root.join(DIRECTORY)) {
        Err(why) if why.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(why) => return Err(why.into()),
        Ok(_) => check_container(root)?,
    }
    let mut objects = Vec::new();
    let mut cancellations = BTreeMap::new();
    let mut count = 0;
    for entry in fs::read_dir(root.join(DIRECTORY))?.take(MAX_RETAINED_STAGES * 2 + 2) {
        let entry = entry?;
        count += 1;
        if count > MAX_RETAINED_STAGES * 2 + 1 {
            return Err(Error::InvalidStage("portable namespace exceeds budget"));
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| Error::InvalidStage("non-UTF8 portable entry"))?;
        if let Some(id) = name.strip_suffix(".cancel").filter(|s| id_name(s)) {
            cancellations.insert(id.to_owned(), cancellation(root, id)?);
        } else if id_name(&name) {
            owned_dir(root, &entry.path())?;
            objects.push(name);
        } else {
            return Err(Error::InvalidStage("unknown portable namespace entry"));
        }
    }
    // Preflight every unpublished object before removing any controls.
    let mut abandoned = Vec::new();
    for name in objects {
        if cancellations.contains_key(&name) {
            continue;
        }
        if !published(root, &name)? {
            let dir = root.join(target(&name));
            abandoned.push((dir.clone(), controls(root, &dir, true)?));
        }
    }
    for (name, manifest) in cancellations {
        let (_slot, _lease) = store.available_owner(manifest.id()?, held)?;
        finish_cancel(root, &name)?;
    }
    for (dir, files) in abandoned {
        remove_controls(&dir, files)?;
    }
    remove_empty_container(root)
}

#[cfg(all(test, unix))]
mod tests;
