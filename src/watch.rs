use std::ffi::c_void;
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

use crate::config::config_dir;
use crate::config::config_path;
use crate::config::profile_name;
use crate::config::read_config;
use crate::config::store_settings;
use crate::config::Settings;
use crate::consts::DEBOUNCE_MS;
use crate::consts::EffectDef;
use crate::consts::INOTIFY_BUF;
use crate::consts::POLL_INTERVAL_MS;
use crate::consts::REGISTRY;
use crate::effect::any_effect_enabled;
use crate::logging::log_at;
use crate::logging::LogLevel;
use crate::shader::build_shaders;
use crate::shader::store_shaders;
use crate::shader::ShaderArtifacts;

static NOTIFY_FD: OnceLock<i32> = OnceLock::new();
static DIRTY: AtomicBool = AtomicBool::new(false);
static BUILDING: AtomicBool = AtomicBool::new(false);
static SHUTDOWN: AtomicBool = AtomicBool::new(false);
static WATCHER: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(None);
static BUILDER: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(None);

fn call_inotify_init() -> i32 {
    unsafe { libc::inotify_init1(libc::IN_NONBLOCK) }
}

fn call_inotify_watch(fd: i32, dir: &PathBuf) -> i32 {
    let c = CString::new(dir.to_string_lossy().as_bytes()).unwrap_or_default();
    unsafe { libc::inotify_add_watch(fd, c.as_ptr(), libc::IN_CLOSE_WRITE | libc::IN_MOVED_TO | libc::IN_CREATE) }
}

fn call_inotify_read(fd: i32) -> isize {
    let mut buf = [0u8; INOTIFY_BUF];
    unsafe { libc::read(fd, buf.as_mut_ptr() as *mut c_void, INOTIFY_BUF) }
}

fn call_poll_fd(fd: i32, timeout_ms: i32) -> i32 {
    let mut pfd = libc::pollfd { fd, events: libc::POLLIN, revents: 0 };
    unsafe { libc::poll(&mut pfd, 1, timeout_ms) }
}

fn has_events(result: i32) -> bool {
    result > 0
}

fn call_inotify_drain_all(fd: i32) {
    std::iter::repeat_with(|| call_inotify_read(fd))
        .take_while(|n| *n > 0)
        .for_each(drop);
}

fn fd_is_valid(fd: i32) -> bool {
    fd >= 0
}

fn reload_is_broken(a: &ShaderArtifacts, s: &Settings, reg: &[EffectDef]) -> bool {
    a.frag.is_empty() && a.comp.is_empty() && any_effect_enabled(s, reg)
}

fn poll_dirty() -> bool {
    DIRTY.swap(false, Ordering::Relaxed)
}

fn shutdown_requested() -> bool {
    SHUTDOWN.load(Ordering::Relaxed)
}

fn watch_step(fd: i32) {
    match has_events(call_poll_fd(fd, POLL_INTERVAL_MS)) {
        true => {
            call_inotify_drain_all(fd);
            thread::sleep(Duration::from_millis(DEBOUNCE_MS));
            call_inotify_drain_all(fd);
            DIRTY.store(true, Ordering::Relaxed);
        }
        false => (),
    }
}

fn watch_loop(fd: i32) {
    std::iter::repeat(())
        .take_while(|_| !shutdown_requested())
        .for_each(|_| watch_step(fd));
}

fn store_handle(slot: &Mutex<Option<thread::JoinHandle<()>>>, h: thread::JoinHandle<()>) {
    match slot.lock() {
        Ok(mut g) => *g = Some(h),
        Err(_) => (),
    }
}

fn take_handle(slot: &Mutex<Option<thread::JoinHandle<()>>>) -> Option<thread::JoinHandle<()>> {
    slot.lock().ok().and_then(|mut g| g.take())
}

fn join_handle(h: thread::JoinHandle<()>) {
    let _ = h.join();
}

fn start_watcher(fd: i32) {
    match fd_is_valid(call_inotify_watch(fd, &config_dir())) {
        true => store_handle(&WATCHER, thread::spawn(move || watch_loop(fd))),
        false => log_at(LogLevel::Warn, "inotify add watch failed, hot reload disabled"),
    }
}

fn init_inotify() {
    NOTIFY_FD.get_or_init(|| {
        let fd = call_inotify_init();
        match fd_is_valid(fd) {
            true => { start_watcher(fd); fd }
            false => fd,
        }
    });
}

fn apply_reload(s: Settings, a: ShaderArtifacts, reg: &[EffectDef]) {
    match reload_is_broken(&a, &s, reg) {
        true => log_at(LogLevel::Warn, "hot reload: spirv compile failed, vulkan postfx will not update"),
        false => (),
    }
    store_shaders(a);
    store_settings(s);
    log_at(LogLevel::Info, "hot reload applied");
}

fn call_close_fd(fd: i32) {
    unsafe { libc::close(fd) };
}

fn close_notify_fd() {
    NOTIFY_FD
        .get()
        .copied()
        .filter(|fd| fd_is_valid(*fd))
        .into_iter()
        .for_each(call_close_fd);
}

fn drain_threads() {
    take_handle(&WATCHER).into_iter().for_each(join_handle);
    take_handle(&BUILDER).into_iter().for_each(join_handle);
}

pub(crate) fn maybe_shutdown_watch(last_instance: bool) {
    match last_instance {
        true => {
            SHUTDOWN.store(true, Ordering::Relaxed);
            drain_threads();
            close_notify_fd();
        }
        false => (),
    }
}

pub(crate) fn setup_watch() {
    init_inotify();
}

fn try_begin_build() -> bool {
    !shutdown_requested() && !BUILDING.swap(true, Ordering::Relaxed)
}

fn end_build() {
    BUILDING.store(false, Ordering::Relaxed);
}

fn requeue_dirty() {
    DIRTY.store(true, Ordering::Relaxed);
}

fn run_reload_build() {
    let s = read_config(&config_path(&profile_name()), &REGISTRY);
    let a = build_shaders(&s, &REGISTRY);
    apply_reload(s, a, &REGISTRY);
    end_build();
}

fn spawn_reload_build() {
    store_handle(&BUILDER, thread::spawn(run_reload_build));
}

fn begin_or_requeue() {
    match try_begin_build() {
        true => spawn_reload_build(),
        false => requeue_dirty(),
    }
}

pub(crate) fn maybe_reload() {
    match poll_dirty() {
        true => begin_or_requeue(),
        false => (),
    }
}
