# 虛擬記憶體 — vm.rs

xv8 使用 Sv39 分頁機制提供程序隔離和虛�擬記憶體管理。

## Sv39 分頁機制

Sv39 是 RISC-V 的 39 位元虛擬位址分頁方案：

```
虛擬位址 (39 bits):
┌─────────────────────┬───────────┬───────────┐
│      VPN[2]         │  VPN[1]   │  VPN[0]   │
│      (9 bits)       │  (9 bits) │  (9 bits) │
├─────────────────────┴───────────┴───────────┴─┼───┤
│                  剩餘 12 bits (offset)         │ PPN│
└─────────────────────────────────────────────────┴───┘

頁表條目 (PTE, 64 bits):
┌────────────────────────────────────────┬─────┬─────┬─────┬─────┐
│         Physical Page Number (PPN)      │ RSW │  U  │  W  │  R  │
│              (44 bits)                  │  2  │  1  │  1  │  1  │
└────────────────────────────────────────┴─────┴─────┴─────┴─────┘
```

## 關鍵結構

```rust
// 實體位址
pub struct PA(usize);

// 虛擬位址
pub struct VA(usize);

// 頁表條目
pub struct PageTableEntry(usize);

// 三層頁表
struct RawPageTable([PageTableEntry; 512]);

// 使用者頁表
pub struct Uvm(pub PageTable);

// 核心頁表
pub struct Kvm(PageTable);
```

## PTE 標誌

```rust
const PTE_V: usize = 1 << 0;  // Valid
const PTE_R: usize = 1 << 1;  // Readable
const PTE_W: usize = 1 << 2;  // Writable
const PTE_X: usize = 1 << 3;  // Executable
const PTE_U: usize = 1 << 4;  // User accessible
const PTE_COW: usize = 1 << 8;  // Copy-on-write
```

## 頁表遍歷

```rust
fn walk_raw(pagetable: NonNull<RawPageTable>, va: VA, alloc: bool)
    -> Result<*mut PageTableEntry, VmError> {

    // 三層遍歷：VPN[2] → VPN[1] → VPN[0]
    for level in (1..=2).rev() {
        let pte = pagetable.as_mut().get_mut(va.px(level));

        if pte.is_v() {
            // 已經映射，繼續向下
            pagetable = NonNull::new(pte.as_pa().as_mut_ptr()).unwrap();
        } else {
            if !alloc {
                err!(VmError::InvalidPage);
            }
            // 建立新的下一層頁表
            pagetable = RawPageTable::try_new()?;
            *pte = PA::from(pagetable.as_ptr() as usize).as_pte() | PTE_V;
        }
    }

    Ok(pagetable.as_mut().get_mut(va.px(0)).unwrap())
}
```

## 記憶體映射

```rust
pub fn map_pages(&mut self, va: VA, pa: PA, size: usize, perm: usize) {
    assert!(va % PGSIZE == 0);
    assert!(size % PGSIZE == 0);

    let last = va + size - PGSIZE;
    let mut va = va;
    let mut pa = pa;

    loop {
        let pte = self.walk_mut(va, true)?;
        assert!(!pte.is_v(), "remap");

        *pte = pa.as_pte() | perm | PTE_V;

        if va == last { break; }
        va += PGSIZE;
        pa += PGSIZE;
    }
}
```

## 核心頁表初始化

```rust
fn make(&mut self) {
    // UART 寄存器
    self.map(VA::from(UART0), PA::from(UART0), PGSIZE, PTE_R | PTE_W);

    // VirtIO 磁碟
    self.map(VA::from(VIRTIO0), PA::from(VIRTIO0), PGSIZE, PTE_R | PTE_W);

    // PCI ECAM (配置空間)
    self.map(VA::from(PCI_ECAM), PA::from(PCI_ECAM), 0x1000_0000, PTE_R | PTE_W);

    // PLIC (中斷控制器)
    self.map(VA::from(PLIC), PA::from(PLIC), 0x400_0000, PTE_R | PTE_W);

    // 核心文字段 (只讀、可執行)
    self.map(VA::from(KERNBASE), PA::from(KERNBASE), etext - KERNBASE, PTE_R | PTE_X);

    // 核心資料段 + 實體記憶體
    self.map(VA::from(etext), PA::from(etext), PHYSTOP - etext, PTE_R | PTE_W);

    // Trampoline (使用者/核心過渡)
    self.map(VA::from(TRAMPOLINE), PA::from(trampoline), PGSIZE, PTE_R | PTE_X);

    // 程序核心堆疊
    unsafe { PROC_TABLE.map_stacks(self) };
}
```

## 使用者頁表操作

### 配置新程序

```rust
pub fn create_pagetable(&self) -> Result<Uvm, VmError> {
    let mut uvm = Uvm::try_new()?;

    // 映射 trampoline (核心用)
    uvm.map_pages(
        TRAMPOLINE.into(),
        (trampoline as *const () as usize).into(),
        PGSIZE,
        PTE_R | PTE_X,
    )?;

    // 映射 trapframe
    uvm.map_pages(
        TRAPFRAME.into(),
        PA::from(data.trapframe() as *const _ as usize),
        PGSIZE,
        PTE_R | PTE_W,
    )?;

    Ok(uvm)
}
```

### 記憶體配置

```rust
pub fn alloc(&mut self, old_size: usize, new_size: usize, xperm: usize)
    -> Result<usize, VmError> {

    let old_size = pg_round_up(old_size);

    for i in (old_size..new_size).step_by(PGSIZE) {
        // 配置實體頁
        let mem = Box::<Page>::try_new_zeroed()?;
        let mem_ptr = Box::into_raw(mem);

        // 映射到虛擬位址
        self.map_pages(
            VA::from(i),
            PA::from(mem_ptr as usize),
            PGSIZE,
            PTE_R | PTE_U | xperm,
        )?;
    }

    Ok(new_size)
}
```

### 解除映射

```rust
pub fn unmap(&mut self, va: VA, npages: usize, free: bool) {
    for i in (va.0..va.0 + (npages * PGSIZE)).step_by(PGSIZE) {
        let pte = self.walk_mut(VA::from(i), false)?;

        if !pte.is_v() {
            continue;  // 懶惰配置，未實際配置
        }

        if free {
            // 釋放實體頁
            let pa = pte.as_pa();
            drop(unsafe { Box::from_raw(pa.as_mut_ptr::<Page>()) });
        }

        *pte = PageTableEntry(0);
    }
}
```

## Copy-on-Write Fork

```rust
pub fn copy(&mut self, child: &mut Uvm, size: usize) -> Result<(), VmError> {
    for i in (0..size).step_by(PGSIZE) {
        let pte = self.walk_mut(VA::from(i), false)?;

        if !pte.is_v() {
            continue;  // 懶惰配置
        }

        // 如果可寫，轉換為 COW
        if pte.is_w() {
            *pte &= !PTE_W;
            *pte |= PTE_COW;
        }

        // 映射到同一個實體頁
        child.map_pages(VA::from(i), pte.as_pa(), PGSIZE, pte.flags())?;

        // 增加參考計數
        kalloc::increment_ref(pte.as_pa());

        // 刷新 TLB
        unsafe { vma::sfence() };
    }
    Ok(())
}
```

## 頁面錯誤處理

```rust
pub fn vmfault(&mut self, va: VA) -> Result<PA, VmError> {
    // 超出邊界
    if va >= data.size {
        err!(VmError::InvalidAddress);
    }

    let va = va.round_down();

    // COW 頁面：需要複製
    if pte.is_cow() {
        let old_pa = pte.as_pa();

        // 配置新頁
        let mem = Box::<Page>::try_new_zeroed()?;
        let new_pa = PA::from(Box::into_raw(mem) as usize);

        // 複製內容
        unsafe {
            ptr::copy_nonoverlapping(
                old_pa.as_mut_ptr(),
                new_pa.as_mut_ptr(),
                PGSIZE,
            );
        }

        // 安裝新頁面（可寫）
        *pte = new_pa.as_pte() | PTE_W | PTE_R | PTE_U & !PTE_COW;

        // 減少舊頁參考
        drop(unsafe { Box::from_raw(old_pa.as_mut_ptr()) });

        return Ok(new_pa);
    }

    // 懶惰配置：首次訪問時配置頁面
    let mem = Box::<Page>::try_new_zeroed()?;
    self.map_pages(va, PA::from(Box::into_raw(mem) as usize), PGSIZE, PTE_W | PTE_U | PTE_R)?;
    Ok(pa)
}
```

## 使用者/核心資料傳輸

```rust
pub fn copy_to(&mut self, src: &[u8], dst: VA) -> Result<(), VmError> {
    let mut dstva = dst.as_usize();

    while !src.is_empty() {
        let va0 = pg_round_down(dstva);

        // 解析並觸發頁面錯誤（如果需要）
        let pa0 = match self.walk_addr(VA::from(va0)) {
            Ok(pa) => pa,
            Err(_) => self.vmfault(VA::from(va0))?,
        };

        let n = (PGSIZE - (dstva - va0)).min(src.len());

        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), (pa0.0 + (dstva - va0)) as *mut u8, n);
        }

        src = &src[n..];
        dstva = va0 + PGSIZE;
    }
    Ok(())
}

pub fn copy_from(&mut self, src: VA, dst: &mut [u8]) -> Result<(), VmError> {
    let mut srcva = src.as_usize();

    while !dst.is_empty() {
        let va0 = pg_round_down(srcva);

        let pa0 = match self.walk_addr(VA::from(va0)) {
            Ok(pa) => pa,
            Err(_) => self.vmfault(VA::from(va0))?,
        };

        let n = (PGSIZE - (srcva - va0)).min(dst.len());

        unsafe {
            ptr::copy_nonoverlapping((pa0.0 + (srcva - va0)) as *const u8, dst.as_mut_ptr(), n);
        }

        dst = &mut dst[n..];
        srcva = va0 + PGSIZE;
    }
    Ok(())
}
```

## 分頁硬體操作

```rust
// 刷新 TLB
pub unsafe fn sfence() {
    asm!("sfence.vma");
}

// 寫入 SATP 寄存器啟用分頁
pub unsafe fn init_hart() {
    sfence();
    satp::write(satp::make(kvm.as_pa().as_usize()));
    sfence();
}
```

## 記憶體配置 (kalloc)

使用 Buddy 配置器分配實體頁：

```rust
static KMEM: Kmem = Kmem(SpinLock::new(None, "kmem"));

unsafe impl GlobalAlloc for Kmem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.0.lock().as_mut().expect("kmem init").malloc(layout.size());

        if !ptr.is_null() {
            // 設定參考計數為 1
            PAGE_REFS[(ptr as usize - KERNBASE) / PGSIZE].store(1, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // 如果是最後一個參考，釋放頁面
        if PAGE_REFS[(ptr as usize - KERNBASE) / PGSIZE].fetch_sub(1, Ordering::Relaxed) == 1 {
            self.0.lock().as_mut().unwrap().free(ptr);
        }
    }
}
```

## 相關主題

- [[Sv39]]：RISC-V 分頁機制
- [[Process]]：程序管理
- [[Trap]]：頁面錯誤處理
- [[Boot]]：記憶體初始化