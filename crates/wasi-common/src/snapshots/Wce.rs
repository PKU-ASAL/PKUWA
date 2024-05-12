use crate::{
    dir::{DirCaps, DirEntry, DirEntryExt, DirFdStat, ReaddirCursor, ReaddirEntity, TableDirExt},
    file::{
        Advice, FdFlags, FdStat, FileCaps, FileEntry, FileEntryExt, FileType, Filestat, OFlags,
        RiFlags, RoFlags, SdFlags, SiFlags, TableFileExt, WasiFile,
    },
    sched::{
        subscription::{RwEventFlags, SubscriptionResult},
        Poll, Userdata,
    },
    Error, ErrorExt, ErrorKind, SystemTimeSpec, WasiCtx,
};
use anyhow::Context;
use cap_std::time::{Duration, SystemClock};
use std::convert::{TryFrom, TryInto};
use std::io::{IoSlice, IoSliceMut};
use std::ops::{Deref, DerefMut};
use tracing::debug;
use wiggle::GuestPtr;

wiggle::from_witx!({
    witx: ["$WASI_ROOT/phases/snapshot/witx/wasi_snapshot_preview1.witx"],
    errors: { errno => Error },
    // Note: not every function actually needs to be async, however, nearly all of them do, and
    // keeping that set the same in this macro and the wasmtime_wiggle / lucet_wiggle macros is
    // tedious, and there is no cost to having a sync function be async in this case.
    // async: *,
    async: {wasi_snapshot_preview1::fd_fdstat_get, wasi_snapshot_preview1::fd_filestat_get, wasi_snapshot_preview1::fd_filestat_set_times, wasi_snapshot_preview1::fd_readdir, wasi_snapshot_preview1::path_create_directory, wasi_snapshot_preview1::path_filestat_get, wasi_snapshot_preview1::path_filestat_set_times, wasi_snapshot_preview1::path_link, wasi_snapshot_preview1::path_open, wasi_snapshot_preview1::path_readlink, wasi_snapshot_preview1::path_remove_directory, wasi_snapshot_preview1::path_rename, wasi_snapshot_preview1::path_symlink, wasi_snapshot_preview1::path_unlink_file, wasi_snapshot_preview1::poll_oneoff, wasi_snapshot_preview1::sched_yield, wasi_snapshot_preview1::sock_accept, wasi_snapshot_preview1::sock_recv, wasi_snapshot_preview1::sock_send, wasi_snapshot_preview1::sock_shutdown},
    wasmtime: false
});
impl wasi_snapshot_preview1::WasiSnapshotPreview1 for WasiCtx {}
