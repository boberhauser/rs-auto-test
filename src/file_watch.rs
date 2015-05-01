//! Module that lets you setup notificiations for file changes in a given directory.
//!
//! Uses the INotify API internal and will therefore only work on Linux.

use std::io;
use std::path::Path;

extern crate inotify;
use self::inotify::INotify;
use self::inotify::wrapper::{Event, Watch};
use self::inotify::ffi::*;

pub const WATCH_MODIFY: i32 = IN_MODIFY as i32;
pub const WATCH_CREATE: i32 = IN_CREATE as i32;
pub const WATCH_DELETE: i32 = IN_DELETE as i32;

/// Can setup multiple notifications for file changes in given directories.
///
/// Will safely remove notifications in the destructor.
pub struct FileWatch {
    notify: INotify,
}

impl FileWatch {
    /// Create a new FileWatch and initialize internal INotify.
    pub fn new() -> io::Result<FileWatch> {
        INotify::init().map(
            |ino: INotify| { FileWatch { notify: ino, } }
        )
    }

    /// Watch the given directory for changes in mask.
    pub fn add_watch(&mut self, path: &Path, mask: i32) -> io::Result<Watch> {
        self.notify.add_watch(path, mask as u32)
    }

    /// Pause current thread until file change events occur.
    pub fn wait_for_events(&mut self) -> io::Result<&[Event]> {
        self.notify.wait_for_events()
    }
}

impl Drop for FileWatch {
    fn drop(&mut self) {
        let _ = self.notify.rm_watch(WATCH_MODIFY | WATCH_CREATE | WATCH_DELETE);
    }
}
