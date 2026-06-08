use core::fmt::Write;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

use crate::proc;
use crate::spinlock::SpinLock;
use crate::sync::OnceLock;
use crate::syscall::SysError;
use crate::vm::VA;

pub const CGROUP_DEV: usize = 2;
const MAX_CGROUPS: usize = 16;

#[derive(Debug, Clone, Copy)]
pub struct CpuController {
    pub max: usize,
    pub period: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryController {
    pub max: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct PidsController {
    pub max: usize,
}

#[derive(Debug, Clone)]
pub struct Cgroup {
    pub id: usize,
    pub name: String,
    pub procs: Vec<usize>,
    pub cpu: CpuController,
    pub memory: MemoryController,
    pub pids: PidsController,
}

struct CgroupTreeInner {
    cgroups: Vec<Option<Cgroup>>,
}

static CGROUP_TREE: OnceLock<SpinLock<CgroupTreeInner>> = OnceLock::new();

pub fn init() {
    let mut cgroups = Vec::with_capacity(MAX_CGROUPS);
    for _ in 0..MAX_CGROUPS {
        cgroups.push(None);
    }
    cgroups[0] = Some(Cgroup {
        id: 0,
        name: String::from("root"),
        procs: Vec::new(),
        cpu: CpuController { max: 0, period: 100000 },
        memory: MemoryController { max: 0 },
        pids: PidsController { max: 0 },
    });
    CGROUP_TREE.initialize::<_, ()>(|| {
        Ok(SpinLock::new(CgroupTreeInner { cgroups }, "cgroup_tree"))
    });
}

fn tree() -> &'static SpinLock<CgroupTreeInner> {
    CGROUP_TREE.get().expect("cgroup tree not initialized")
}

fn find_slot(inner: &CgroupTreeInner) -> Option<usize> {
    inner.cgroups.iter().position(|cg| cg.is_none())
}

fn find_by_name<'a>(inner: &'a CgroupTreeInner, name: &str) -> Option<usize> {
    inner
        .cgroups
        .iter()
        .position(|cg| cg.as_ref().is_some_and(|c| c.name == name))
}

fn parse_cmd(cmd: &str) -> Result<(), &'static str> {
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return Ok(());
    }
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }
    match parts[0] {
        "create" => {
            if parts.len() < 2 {
                return Err("usage: create <name>");
            }
            let name = parts[1];
            let mut tree = tree().lock();
            if find_by_name(&tree, name).is_some() {
                return Err("cgroup already exists");
            }
            let slot = find_slot(&tree).ok_or("no free cgroup slots")?;
            tree.cgroups[slot] = Some(Cgroup {
                id: slot,
                name: String::from(name),
                procs: Vec::new(),
                cpu: CpuController { max: 0, period: 100000 },
                memory: MemoryController { max: 0 },
                pids: PidsController { max: 0 },
            });
            Ok(())
        }
        "attach" => {
            if parts.len() < 3 {
                return Err("usage: attach <pid> <name>");
            }
            let pid: usize = parts[1].parse().map_err(|_| "invalid pid")?;
            let name = parts[2];
            let mut tree = tree().lock();
            let cg_idx =
                find_by_name(&tree, name).ok_or("cgroup not found")?;
            if let Some(ref mut cg) = tree.cgroups[cg_idx] {
                if !cg.procs.contains(&pid) {
                    cg.procs.push(pid);
                }
            }
            Ok(())
        }
        "cpu.max" => {
            if parts.len() < 4 {
                return Err("usage: cpu.max <max> <period> <name>");
            }
            let max: usize = parts[1].parse().map_err(|_| "invalid max")?;
            let period: usize = parts[2].parse().map_err(|_| "invalid period")?;
            let name = parts[3];
            let mut tree = tree().lock();
            let cg_idx =
                find_by_name(&tree, name).ok_or("cgroup not found")?;
            if let Some(ref mut cg) = tree.cgroups[cg_idx] {
                cg.cpu = CpuController { max, period };
            }
            Ok(())
        }
        "memory.max" => {
            if parts.len() < 3 {
                return Err("usage: memory.max <max> <name>");
            }
            let max: usize = parts[1].parse().map_err(|_| "invalid max")?;
            let name = parts[2];
            let mut tree = tree().lock();
            let cg_idx =
                find_by_name(&tree, name).ok_or("cgroup not found")?;
            if let Some(ref mut cg) = tree.cgroups[cg_idx] {
                cg.memory = MemoryController { max };
            }
            Ok(())
        }
        "pids.max" => {
            if parts.len() < 3 {
                return Err("usage: pids.max <max> <name>");
            }
            let max: usize = parts[1].parse().map_err(|_| "invalid max")?;
            let name = parts[2];
            let mut tree = tree().lock();
            let cg_idx =
                find_by_name(&tree, name).ok_or("cgroup not found")?;
            if let Some(ref mut cg) = tree.cgroups[cg_idx] {
                cg.pids = PidsController { max };
            }
            Ok(())
        }
        _ => Err("unknown command"),
    }
}

pub fn device_write(src: VA, len: usize) -> Result<usize, SysError> {
    let mut buf = alloc::vec![0u8; len];
    if proc::copy_from_user(src, &mut buf).is_err() {
        return Err(SysError::BadAddress);
    }
    let cmd = core::str::from_utf8(&buf).map_err(|_| SysError::InvalidArgument)?;
    if parse_cmd(cmd).is_err() {
        return Err(SysError::InvalidArgument);
    }
    Ok(len)
}

fn stats_string() -> String {
    let tree = tree().lock();
    let mut s = String::new();
    for cg_opt in tree.cgroups.iter() {
        if let Some(cg) = cg_opt {
            let _ = write!(
                s,
                "cgroup {}\n  cpu: max={} period={}\n  memory: max={}\n  pids: max={}\n  procs:",
                cg.name, cg.cpu.max, cg.cpu.period, cg.memory.max, cg.pids.max,
            );
            for pid in &cg.procs {
                let _ = write!(s, " {}", pid);
            }
            let _ = writeln!(s);
        }
    }
    s
}

pub fn device_read(dst: VA, len: usize) -> Result<usize, SysError> {
    let stats = stats_string();
    let bytes = stats.as_bytes();
    let n = bytes.len().min(len);
    if proc::copy_to_user(&bytes[..n], dst).is_err() {
        return Err(SysError::BadAddress);
    }
    Ok(n)
}
