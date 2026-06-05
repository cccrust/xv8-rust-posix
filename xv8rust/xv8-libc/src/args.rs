use core::slice;

static mut ARGC: usize = 0;
static mut ARGV: *const *const u8 = core::ptr::null();

pub fn init(argc: usize, argv: *const *const u8) {
    unsafe {
        ARGC = argc;
        ARGV = argv;
    }
}

pub struct Args {
    pub argc: usize,
    pub argv: *const *const u8,
}

pub struct ArgsIter {
    pub argv: *const *const u8,
    pub current: usize,
    pub end: usize,
}

impl Args {
    #[inline(always)]
    pub unsafe fn from_stack() -> Self {
        Self { argc: ARGC, argv: ARGV }
    }

    pub fn len(&self) -> usize {
        self.argc
    }

    pub fn get(&self, index: usize) -> Option<&'static [u8]> {
        if index >= self.argc {
            return None;
        }
        unsafe {
            let ptr = *self.argv.add(index);
            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            Some(slice::from_raw_parts(ptr, len))
        }
    }

    pub fn get_str(&self, index: usize) -> Option<&'static str> {
        self.get(index).and_then(|arg| core::str::from_utf8(arg).ok())
    }

    pub fn args(&self) -> ArgsIter {
        ArgsIter {
            argv: self.argv,
            current: 1,
            end: self.argc,
        }
    }
}

impl Iterator for ArgsIter {
    type Item = &'static [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }
        unsafe {
            let ptr = *self.argv.add(self.current);
            self.current += 1;
            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            Some(slice::from_raw_parts(ptr, len))
        }
    }
}
