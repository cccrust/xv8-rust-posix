use crate::process::Command;

pub trait CommandExt {
    fn uid(&mut self, _id: u32) -> &mut Command;
    fn gid(&mut self, _id: u32) -> &mut Command;
}

impl CommandExt for Command {
    fn uid(&mut self, _id: u32) -> &mut Command { self }
    fn gid(&mut self, _id: u32) -> &mut Command { self }
}
