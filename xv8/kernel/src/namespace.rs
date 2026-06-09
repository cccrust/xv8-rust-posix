use core::fmt;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::spinlock::SpinLock;
use crate::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NsType {
    Mount = 0,
    Cgroup = 1,
    Uts = 2,
    Ipc = 3,
    User = 4,
    Pid = 5,
    Net = 6,
}

pub fn nstype_to_flag(t: NsType) -> usize {
    match t {
        NsType::Mount => CLONE_NEWNS,
        NsType::Cgroup => CLONE_NEWCGROUP,
        NsType::Uts => CLONE_NEWUTS,
        NsType::Ipc => CLONE_NEWIPC,
        NsType::User => CLONE_NEWUSER,
        NsType::Pid => CLONE_NEWPID,
        NsType::Net => CLONE_NEWNET,
    }
}

// ── CLONE_NEW* flag constants ──────────────────────────────────────────────

pub const CLONE_NEWNS: usize = 0x00020000;
pub const CLONE_NEWCGROUP: usize = 0x02000000;
pub const CLONE_NEWUTS: usize = 0x04000000;
pub const CLONE_NEWIPC: usize = 0x08000000;
pub const CLONE_NEWUSER: usize = 0x10000000;
pub const CLONE_NEWPID: usize = 0x20000000;
pub const CLONE_NEWNET: usize = 0x40000000;

pub const CLONE_NEW_ALL: usize =
    CLONE_NEWNS | CLONE_NEWCGROUP | CLONE_NEWUTS | CLONE_NEWIPC
    | CLONE_NEWUSER | CLONE_NEWPID | CLONE_NEWNET;

// ── NamespaceId (global unique counter) ────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NamespaceId(usize);

impl NamespaceId {
    fn alloc() -> Self {
        static NS_COUNT: AtomicUsize = AtomicUsize::new(1);
        NamespaceId(NS_COUNT.fetch_add(1, Ordering::Relaxed))
    }
}

// ── UTS namespace ──────────────────────────────────────────────────────────

const MAX_HOSTNAME_LEN: usize = 64;

pub struct UtsData {
    hostname: [u8; MAX_HOSTNAME_LEN],
    hostname_len: usize,
}

impl UtsData {
    const fn new() -> Self {
        UtsData { hostname: [0u8; MAX_HOSTNAME_LEN], hostname_len: 0 }
    }

    pub fn set_hostname(&mut self, name: &[u8]) -> Result<(), ()> {
        if name.len() > MAX_HOSTNAME_LEN || name.is_empty() {
            return Err(());
        }
        self.hostname = [0u8; MAX_HOSTNAME_LEN];
        self.hostname[..name.len()].copy_from_slice(name);
        self.hostname_len = name.len();
        Ok(())
    }

    pub fn hostname(&self) -> &[u8] {
        &self.hostname[..self.hostname_len]
    }
}

pub struct UtsNamespace {
    pub id: NamespaceId,
    pub data: SpinLock<UtsData>,
}

impl fmt::Debug for UtsNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UtsNamespace").field("id", &self.id).finish()
    }
}

impl UtsNamespace {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            id: NamespaceId::alloc(),
            data: SpinLock::new(UtsData::new(), "uts"),
        })
    }

    pub fn clone_new(&self) -> Arc<Self> {
        let data = self.data.lock();
        let mut new_uts = UtsData::new();
        let _ = new_uts.set_hostname(data.hostname());
        drop(data);
        Arc::new(Self {
            id: NamespaceId::alloc(),
            data: SpinLock::new(new_uts, "uts"),
        })
    }
}

// ── PID namespace ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct PidNamespace {
    pub id: NamespaceId,
    next_ns_pid: AtomicUsize,
}

impl PidNamespace {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            id: NamespaceId::alloc(),
            next_ns_pid: AtomicUsize::new(1),
        })
    }

    pub fn alloc_ns_pid(&self) -> crate::proc::Pid {
        // SAFETY: the next_ns_pid counter starts at 1 and monotonically increases,
        // producing unique values that never overlap with Pid::alloc().
        unsafe { crate::proc::Pid::from_usize(self.next_ns_pid.fetch_add(1, Ordering::Relaxed)) }
    }
}

// ── Placeholder namespace types ────────────────────────────────────────────

#[derive(Debug)]
pub struct MountNamespace {
    pub id: NamespaceId,
}

impl MountNamespace {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { id: NamespaceId::alloc() })
    }
}

#[derive(Debug)]
pub struct NetNamespace {
    pub id: NamespaceId,
}

impl NetNamespace {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { id: NamespaceId::alloc() })
    }
}

#[derive(Debug)]
pub struct IpcNamespace {
    pub id: NamespaceId,
}

impl IpcNamespace {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { id: NamespaceId::alloc() })
    }
}

#[derive(Debug)]
pub struct UserNamespace {
    pub id: NamespaceId,
}

impl UserNamespace {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { id: NamespaceId::alloc() })
    }
}

#[derive(Debug)]
pub struct CgroupNamespace {
    pub id: NamespaceId,
}

impl CgroupNamespace {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { id: NamespaceId::alloc() })
    }
}

// ── NsProxy: one per process (stores Arc to each namespace type) ───────────

#[derive(Debug)]
pub struct NsProxy {
    pub pid: Arc<PidNamespace>,
    pub uts: Arc<UtsNamespace>,
    pub mount: Arc<MountNamespace>,
    pub net: Arc<NetNamespace>,
    pub ipc: Arc<IpcNamespace>,
    pub user: Arc<UserNamespace>,
    pub cgroup: Arc<CgroupNamespace>,
}

impl Clone for NsProxy {
    fn clone(&self) -> Self {
        Self {
            pid: self.pid.clone(),
            uts: self.uts.clone(),
            mount: self.mount.clone(),
            net: self.net.clone(),
            ipc: self.ipc.clone(),
            user: self.user.clone(),
            cgroup: self.cgroup.clone(),
        }
    }
}

impl Default for NsProxy {
    fn default() -> Self {
        Self::new()
    }
}

impl NsProxy {
    pub fn new() -> Self {
        Self {
            pid: PidNamespace::new(),
            uts: UtsNamespace::new(),
            mount: MountNamespace::new(),
            net: NetNamespace::new(),
            ipc: IpcNamespace::new(),
            user: UserNamespace::new(),
            cgroup: CgroupNamespace::new(),
        }
    }

    /// Returns a new NsProxy with one namespace type replaced by the given source's.
    pub fn clone_with_override(&self, nstype: NsType, source: &Self) -> Self {
        let mut ns = self.clone();
        match nstype {
            NsType::Pid => ns.pid = source.pid.clone(),
            NsType::Uts => ns.uts = source.uts.clone(),
            NsType::Mount => ns.mount = source.mount.clone(),
            NsType::Net => ns.net = source.net.clone(),
            NsType::Ipc => ns.ipc = source.ipc.clone(),
            NsType::User => ns.user = source.user.clone(),
            NsType::Cgroup => ns.cgroup = source.cgroup.clone(),
        }
        ns
    }

    pub fn from_parent(parent: &NsProxy, flags: usize) -> Self {
        Self {
            pid: if flags & CLONE_NEWPID != 0 {
                PidNamespace::new()
            } else {
                parent.pid.clone()
            },
            uts: if flags & CLONE_NEWUTS != 0 {
                parent.uts.clone_new()
            } else {
                parent.uts.clone()
            },
            mount: if flags & CLONE_NEWNS != 0 {
                MountNamespace::new()
            } else {
                parent.mount.clone()
            },
            net: if flags & CLONE_NEWNET != 0 {
                NetNamespace::new()
            } else {
                parent.net.clone()
            },
            ipc: if flags & CLONE_NEWIPC != 0 {
                IpcNamespace::new()
            } else {
                parent.ipc.clone()
            },
            user: if flags & CLONE_NEWUSER != 0 {
                UserNamespace::new()
            } else {
                parent.user.clone()
            },
            cgroup: if flags & CLONE_NEWCGROUP != 0 {
                CgroupNamespace::new()
            } else {
                parent.cgroup.clone()
            },
        }
    }
}

// ── Root namespaces (initialized once at boot) ────────────────────────────

static ROOT_NS: OnceLock<NsProxy> = OnceLock::new();

pub fn init_root() {
    ROOT_NS.initialize::<_, ()>(|| Ok(NsProxy::new()));
}

pub fn root_ns() -> &'static NsProxy {
    ROOT_NS.get().expect("root namespace not initialized")
}
