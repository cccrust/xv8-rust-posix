# ELF 執行 — exec.rs

exec 系統呼叫載入並執行 ELF 格式的可執行檔案。

## ELF 格式結構

```rust
struct ElfHeader {
    magic: u32,           // 0x464C457F ("\x7FELF")
    elf: [u8; 12],
    type: u16,            // 可執行檔類型
    machine: u16,         // RISC-V
    version: u32,
    entry: u64,           // 入口點位址
    phoff: u64,           // 程式標題偏移
    shoff: u64,           // 區段標題偏移
    flags: u32,
    ehsize: u16,          // ELF 標頭大小
    phentsize: u16,       // 程式標題大小
    phnum: u16,           // 程式標題數量
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

struct ProgramHeader {
    type: u32,            // PT_LOAD = 1
    flags: u32,          // 0x1=EXEC, 0x2=WRITE, 0x4=READ
    offset: u64,
    vaddr: u64,
    paddr: u64,
    filesz: u64,
    memsz: u64,
    align: u64,
}
```

## exec 流程

```rust
pub fn exec(path: &Path, argv: &[&str]) -> Result<usize, ExecError> {
    let _op = Operation::begin();  // 日誌交易

    // 1. 開啟執行檔
    let mut inode = path.resolve()?;
    let mut inner = inode.lock();

    // 2. 讀取 ELF 標頭
    let mut elf_buf = [0u8; ElfHeader::SIZE];
    inode.read(&mut inner, 0, &mut elf_buf, false)?;
    let elf = ElfHeader::from_bytes(&elf_buf);

    // 3. 驗證 ELF magic
    if elf.magic != ELF_MAGIC {
        err!(ExecError::Elf);
    }

    // 4. 建立新頁表
    let mut pagetable = proc.create_pagetable()?;

    // 5. 載入每個程式段
    for i in 0..elf.phnum {
        let ph = read_program_header(&mut inode, &mut inner, elf.phoff + i * ProgramHeader::SIZE)?;

        if ph.type != ELF_PROG_LOAD {
            continue;
        }

        // 驗證位址
        if ph.memsz < ph.filesz || !is_aligned(ph.vaddr) {
            err!(ExecError::Header);
        }

        // 配置記憶體
        let new_size = pagetable.alloc(size, (ph.vaddr + ph.memsz) as usize, ph.get_perms())?;

        // 載入資料
        pagetable.load_elf_segment(&mut inode, &mut inner,
                                    VA::from(ph.vaddr as usize),
                                    ph.offset as u32,
                                    ph.filesz as usize)?;
    }

    // 6. 配置使用者堆疊
    size = pg_round_up(size);
    size = pagetable.alloc(size, size + (USERSTACK + 1) * PGSIZE, PTE_W)?;

    // 保護頁（不可訪問）
    pagetable.clear(VA::from(size - (USERSTACK + 1) * PGSIZE))?;

    // 7. 複製參數到堆疊
    let mut sp = size;
    let stackbase = sp - USERSTACK * PGSIZE;
    let mut ustack = [0u64; MAXARG];
    let mut argc = 0;

    for &arg in argv.iter() {
        sp -= arg.len() + 1;
        sp -= sp % 16;  // 16 位元組對齊

        // 複製參數字串
        pagetable.copy_to(arg.as_bytes(), VA::from(sp))?;
        pagetable.copy_to(&[0u8], VA::from(sp + arg.len()))?;

        ustack[argc] = sp as u64;
        argc += 1;
    }

    // 8. 複製 argv 指標陣列
    sp -= (argc + 1) * size_of::<u64>();
    sp -= sp % 16;

    pagetable.copy_to(
        unsafe { slice::from_raw_parts(ustack.as_ptr(), (argc + 1) * 8) },
        VA::from(sp)
    )?;

    inode.unlock_put(inner);
    drop(_op);

    // 9. 切換到新頁表
    let old_pagetable = proc.data().pagetable.replace(pagetable).unwrap();
    proc.data().size = size;

    // 10. 設定 trapframe
    let trapframe = proc.data().trapframe_mut();
    trapframe.a1 = sp;                          // argv指標
    trapframe.epc = elf.entry as usize;        // 入口點
    trapframe.sp = sp;                          // 堆疊指標

    // 11. 釋放舊頁表
    old_pagetable.proc_free(old_size);

    Ok(argc)
}
```

## 程式段載入

```rust
pub fn load_elf_segment(&self, inode: &mut Inode, inner: &mut SleepLockGuard<'_, InodeInner>,
                        va: VA, offset: u32, size: usize) -> Result<(), VmError> {
    for i in (0..size).step_by(PGSIZE) {
        // 檢查並配置頁面（如果尚未配置）
        let pa = self.walk_addr(va + i)?;

        let n = if size - i < PGSIZE { size - i } else { PGSIZE };

        // 從磁碟讀取
        let dst = unsafe { core::slice::from_raw_parts_mut(pa.as_usize() as *mut u8, n) };
        inode.read(inner, offset + i as u32, dst, false)?;
    }
    Ok(())
}
```

## 堆疊設置

```
記憶體佈局（高址到低址）：

┌─────────────────────────────────────────┐
│          堆疊保護頁 (不可訪問)          │
├─────────────────────────────────────────┤
│                                         │
│  使用者堆疊                              │
│  ┌─────────────────────────────────┐   │
│  │ argv[0] 字串                     │   │
│  │ argv[1] 字串                     │   │
│  │ ...                              │   │
│  │ argv[argc-1] 字串                │   │
│  │ (null terminator)               │   │
│  ├─────────────────────────────────┤   │
│  │ padding                          │   │
│  ├─────────────────────────────────┤   │
│  │ argv[argc] = 0                   │   │
│  │ argv[argc-1]                    │   │
│  │ ...                              │   │
│  │ argv[0]                          │   │
│  ├─────────────────────────────────┤   │
│  │ padding (16 byte aligned)        │   │
│  ├─────────────────────────────────┤   │
│  │ stack pointer (sp)               │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ... (grows down)                       │
│                                         │
├─────────────────────────────────────────┤
│          未映射區域                      │
├─────────────────────────────────────────┤
│                                         │
│  程式段 (text, data, bss)               │
│                                         │
└─────────────────────────────────────────┘
          0x0
```

## 許可權計算

```rust
impl ProgramHeader {
    fn get_perms(&self) -> usize {
        let mut perm = 0;
        if self.flags & 0x1 != 0 {  // PF_X
            perm |= PTE_X;
        }
        if self.flags & 0x2 != 0 {  // PF_W
            perm |= PTE_W;
        }
        if self.flags & 0x4 != 0 {  // PF_R
            perm |= PTE_R;
        }
        perm
    }
}
```

## 錯誤處理

```rust
enum ExecError {
    Alloc,    // 記憶體配置失敗
    Elf,      // 無效 ELF
    Header,   // 無效程式標頭
    Read,     // 讀取失敗
    Memory,   // 記憶體相關錯誤
}
```

## 相關主題

- [[Process]]：程序管理
- [[Sv39]]：虛擬記憶體
- [[Trap]]：陷阱處理