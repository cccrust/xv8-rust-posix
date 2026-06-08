use alloc::string::String;
use alloc::vec::Vec;

use crate::fs::{Directory, FsError, Inode, InodeType, Path, ROOTINO};
use crate::log::Operation;
use crate::param::ROOTDEV;
use crate::spinlock::SpinLock;
use crate::sync::OnceLock;
use crate::syscall::SysError;

// ── Overlay mount entry ─────────────────────────────────────────────────────

pub struct OverlayMount {
    pub mount_dir_inum: u32,
    pub upper_inum: u32,
    pub lower_inum: u32,
}

// ── Global mount table ─────────────────────────────────────────────────────

pub struct OverlayTable {
    pub mounts: Vec<OverlayMount>,
}

impl OverlayTable {
    pub const fn new() -> Self {
        Self { mounts: Vec::new() }
    }
}

pub static OVERLAY_MOUNTS: OnceLock<SpinLock<OverlayTable>> = OnceLock::new();

pub fn init() {
    OVERLAY_MOUNTS.initialize::<_, ()>(|| {
        Ok(SpinLock::new(OverlayTable::new(), "overlay"))
    });
}

/// Resolve a remaining path under an overlay mount.
/// Called from Path::resolve_inner after crossing a mount point.
/// remaining is the path suffix after the mount point (has lifetime 'a).
pub fn resolve_overlay_str<'a>(
    upper_inum: u32,
    lower_inum: u32,
    remaining: &'a str,
    parent: bool,
) -> Result<(super::fs::Inode, &'a str), super::fs::FsError> {
    let trimmed = remaining.trim_start_matches('/');
    let components: Vec<&str> = trimmed.split('/').filter(|c| !c.is_empty()).collect();

    if components.is_empty() {
        let inode = super::fs::Inode::get(super::param::ROOTDEV, upper_inum)?;
        return Ok((inode, ""));
    }

    let last_name = components.last().copied().unwrap_or("");
    let (inode, _from_upper) = resolve_overlay_components(upper_inum, lower_inum, &components, parent)?;
    Ok((inode, last_name))
}

/// Check if an inode is an overlay mount point, returning (upper_inum, lower_inum)
pub fn find_overlay(inum: u32) -> Option<(u32, u32)> {
    let table = OVERLAY_MOUNTS.get().unwrap().lock();
    for m in table.mounts.iter() {
        if m.mount_dir_inum == inum {
            return Some((m.upper_inum, m.lower_inum));
        }
    }
    None
}

/// Find overlay info by upper or lower inum (used when we know we're inside an overlay)
pub fn find_overlay_by_child(upper_inum: u32, lower_inum: u32) -> Option<(u32, u32)> {
    let table = OVERLAY_MOUNTS.get().unwrap().lock();
    for m in table.mounts.iter() {
        if m.upper_inum == upper_inum || m.lower_inum == lower_inum {
            return Some((m.upper_inum, m.lower_inum));
        }
    }
    None
}

// ── Path resolution helpers ────────────────────────────────────────────────

/// Split a path into components
fn path_components(s: &str) -> Vec<&str> {
    s.trim_start_matches('/')
        .split('/')
        .filter(|c| !c.is_empty())
        .collect()
}

/// Build a string from components
fn join_components(components: &[&str]) -> alloc::string::String {
    let mut s = alloc::string::String::new();
    for (i, c) in components.iter().enumerate() {
        if i > 0 {
            s.push('/');
        }
        s.push_str(c);
    }
    s
}

/// Merged lookup: try upper first, then lower.
/// Returns (inode, from_upper)
fn overlay_lookup(upper_inum: u32, lower_inum: u32, name: &str) -> Result<Option<(Inode, bool)>, FsError> {
    let upper_dir = Inode::get(ROOTDEV, upper_inum)?;
    let mut inner = upper_dir.lock();
    let result = Directory::lookup(&upper_dir, &mut inner, name)?;
    upper_dir.unlock_put(inner);

    if let Some((_, inode)) = result {
        return Ok(Some((inode, true)));
    }

    let lower_dir = Inode::get(ROOTDEV, lower_inum)?;
    let mut inner = lower_dir.lock();
    let result = Directory::lookup(&lower_dir, &mut inner, name)?;
    lower_dir.unlock_put(inner);

    if let Some((_, inode)) = result {
        Ok(Some((inode, false)))
    } else {
        Ok(None)
    }
}

/// Resolve a path under an overlay mount.
/// components[0..] is relative path within the overlay.
/// Returns (inode, from_upper).
fn resolve_overlay_components(
    upper_inum: u32,
    lower_inum: u32,
    components: &[&str],
    parent: bool,
) -> Result<(Inode, bool), FsError> {
    if components.is_empty() || (components.len() == 1 && components[0].is_empty()) {
        // resolve to the mount point itself - return the upper dir
        return Inode::get(ROOTDEV, upper_inum).map(|i| (i, true));
    }

    let len = if parent { components.len() - 1 } else { components.len() };
    if parent && len == 0 {
        // parent of first component is the overlay root (upper dir)
        return Inode::get(ROOTDEV, upper_inum).map(|i| (i, true));
    }

    let mut from_upper = true;
    let mut dir_inum = upper_inum;

    for (i, &name) in components[..len].iter().enumerate() {
        let is_last = i == len - 1;

        match overlay_lookup(dir_inum, lower_inum, name)? {
            Some((inode, upper)) => {
                // drop old dir reference
                if dir_inum != upper_inum && dir_inum != lower_inum {
                    if let Ok(old) = Inode::get(ROOTDEV, dir_inum) {
                        old.put();
                    }
                }
                dir_inum = inode.inum;
                from_upper = upper;

                if is_last {
                    if parent {
                        // return parent (current dir_inum)
                        return Inode::get(ROOTDEV, dir_inum).map(|i| (i, from_upper));
                    }
                    // return the final inode
                    return Ok((inode, upper));
                }
            }
            None => return Err(FsError::Resolve),
        }
    }

    if parent {
        Inode::get(ROOTDEV, dir_inum).map(|i| (i, from_upper))
    } else {
        // Should not reach here
        Err(FsError::Resolve)
    }
}

/// Check if a path is under an overlay mount and resolve it.
pub fn overlay_resolve_path(path: &str) -> Result<(Inode, bool), FsError> {
    let components = path_components(path);
    if components.is_empty() {
        return Err(FsError::Resolve);
    }

    let mut current = Inode::get(ROOTDEV, ROOTINO)?;
    let mut idx = 0;

    loop {
        if idx >= components.len() {
            current.put();
            return Err(FsError::Resolve);
        }

        let name = components[idx];
        let mut inner = current.lock();
        if inner.r#type != InodeType::Directory {
            current.unlock_put(inner);
            return Err(FsError::Resolve);
        }
        match Directory::lookup(&current, &mut inner, name)? {
            Some((_, next)) => {
                current.unlock_put(inner);

                if let Some((up_inum, lo_inum)) = find_overlay(next.inum) {
                    next.put();
                    let remaining = &components[idx + 1..];
                    return resolve_overlay_components(up_inum, lo_inum, remaining, false);
                }

                current = next;
                idx += 1;
            }
            None => {
                current.unlock_put(inner);
                return Err(FsError::Resolve);
            }
        }
    }
}

/// Resolve parent of a path under overlay.
/// Returns (parent_inode, last_component, from_upper).
pub fn overlay_resolve_parent(path: &str) -> Result<(Inode, &str, bool), FsError> {
    let components = path_components(path);
    if components.is_empty() {
        return Err(FsError::Resolve);
    }

    // Walk normally until we hit a mount point
    let mut current = Inode::get(ROOTDEV, ROOTINO)?;
    let mut idx = 0;

    loop {
        if idx >= components.len() {
            current.put();
            return Err(FsError::Resolve);
        }

        let name = components[idx];
        let mut inner = current.lock();
        if inner.r#type != InodeType::Directory {
            current.unlock_put(inner);
            return Err(FsError::Resolve);
        }
        match Directory::lookup(&current, &mut inner, name)? {
            Some((_, next)) => {
                current.unlock_put(inner);

                if let Some((up_inum, lo_inum)) = find_overlay(next.inum) {
                    let last = components.last().copied().unwrap_or("");
                    let parent_comps = &components[idx + 1..components.len() - 1];

                    if parent_comps.is_empty() && last.is_empty() {
                        next.put();
                        let parent = Inode::get(ROOTDEV, up_inum).map_err(|_| FsError::Resolve)?;
                        return Ok((parent, last, true));
                    }

                    next.put();
                    let (parent, from_upper) = resolve_overlay_components(up_inum, lo_inum, &components[idx + 1..], true)?;
                    let last_name = components.last().copied().unwrap_or("");
                    return Ok((parent, last_name, from_upper));
                }

                current = next;
                idx += 1;
            }
            None => {
                current.unlock_put(inner);
                return Err(FsError::Resolve);
            }
        }
    }
}

// ── Create / Unlink helpers ────────────────────────────────────────────────

/// Create a file/directory under an overlay mount point.
/// The path must be under an overlay mount.
pub fn overlay_create(path: &str, r#type: InodeType, major: u16, minor: u16) -> Result<Inode, FsError> {
    let (parent, name, _from_upper) = overlay_resolve_parent(path)?;
    let _op = Operation::begin();

    let mut parent_inner = parent.lock();
    if let Ok(Some((_, inode))) = Directory::lookup(&parent, &mut parent_inner, name) {
        parent.unlock_put(parent_inner);
        let inner = inode.lock();
        if r#type == InodeType::File && (inner.r#type == InodeType::File || inner.r#type == InodeType::Device) {
            return Ok(inode);
        }
        inode.unlock_put(inner);
        return Err(FsError::Create);
    }

    let pdev = parent.dev;
    let pinum = parent.inum;
    let new_inode = match Inode::alloc(pdev, r#type) {
        Ok(i) => i,
        Err(e) => {
            parent.unlock_put(parent_inner);
            return Err(e);
        }
    };

    let mut inode_inner = new_inode.lock();
    inode_inner.major = major;
    inode_inner.minor = minor;
    inode_inner.nlink = 1;
    new_inode.update(&inode_inner);

    if r#type == InodeType::Directory {
        if Directory::link(&new_inode, &mut inode_inner, ".", new_inode.inum as u16).is_err()
            || Directory::link(&new_inode, &mut inode_inner, "..", pinum as u16).is_err()
        {
            inode_inner.nlink = 0;
            new_inode.update(&inode_inner);
            new_inode.unlock_put(inode_inner);
            parent.unlock_put(parent_inner);
            return Err(FsError::Create);
        }
    }

    if Directory::link(&parent, &mut parent_inner, name, new_inode.inum as u16).is_err() {
        inode_inner.nlink = 0;
        new_inode.update(&inode_inner);
        new_inode.unlock_put(inode_inner);
        parent.unlock_put(parent_inner);
        return Err(FsError::Create);
    }

    if r#type == InodeType::Directory {
        parent_inner.nlink += 1;
        parent.update(&parent_inner);
    }

    new_inode.unlock(inode_inner);
    parent.unlock_put(parent_inner);
    Ok(new_inode)
}

// ── Mount / Unmount syscalls ──────────────────────────────────────────────

pub fn sys_mount(mount_point: &str, upper_path: &str, lower_path: &str) -> Result<usize, SysError> {
    let _op = Operation::begin();

    let mp_inode = Path::new(mount_point).resolve().map_err(|_| SysError::InvalidArgument)?;
    let up_inode = Path::new(upper_path).resolve().map_err(|_| SysError::InvalidArgument)?;
    let lo_inode = Path::new(lower_path).resolve().map_err(|_| SysError::InvalidArgument)?;

    let mp_inum = mp_inode.inum;

    // Verify all are directories
    let mp_inner = mp_inode.lock();
    if mp_inner.r#type != InodeType::Directory {
        mp_inode.unlock_put(mp_inner);
        up_inode.put();
        lo_inode.put();
        return Err(SysError::InvalidArgument);
    }
    mp_inode.unlock(mp_inner);

    let up_inner = up_inode.lock();
    if up_inner.r#type != InodeType::Directory {
        up_inode.unlock_put(up_inner);
        mp_inode.put();
        lo_inode.put();
        return Err(SysError::InvalidArgument);
    }
    let up_inum = up_inode.inum;
    up_inode.unlock(up_inner);

    let lo_inner = lo_inode.lock();
    if lo_inner.r#type != InodeType::Directory {
        lo_inode.unlock_put(lo_inner);
        mp_inode.put();
        up_inode.put();
        return Err(SysError::InvalidArgument);
    }
    let lo_inum = lo_inode.inum;
    lo_inode.unlock(lo_inner);

    mp_inode.put();
    up_inode.put();
    lo_inode.put();

    let mut table = OVERLAY_MOUNTS.get().unwrap().lock();
    if table.mounts.iter().any(|m| m.mount_dir_inum == mp_inum) {
        return Err(SysError::AlreadyExists);
    }
    table.mounts.push(OverlayMount {
        mount_dir_inum: mp_inum,
        upper_inum: up_inum,
        lower_inum: lo_inum,
    });
    Ok(0)
}

pub fn sys_umount(mount_point: &str) -> Result<usize, SysError> {
    // Resolve mount point manually to bypass overlay hook in resolve_inner
    let (parent, name) = Path::new(mount_point).resolve_parent()
        .map_err(|_| SysError::InvalidArgument)?;
    let mut inner = parent.lock();
    let mp_inode = match Directory::lookup(&parent, &mut inner, name)
        .map_err(|_| SysError::InvalidArgument)?
    {
        Some((_, inode)) => {
            parent.unlock_put(inner);
            inode
        }
        None => {
            parent.unlock_put(inner);
            return Err(SysError::InvalidArgument);
        }
    };
    let mp_inum = mp_inode.inum;
    mp_inode.put();

    let mut table = OVERLAY_MOUNTS.get().unwrap().lock();
    let pos = table.mounts.iter().position(|m| m.mount_dir_inum == mp_inum)
        .ok_or(SysError::InvalidArgument)?;
    table.mounts.swap_remove(pos);
    Ok(0)
}
