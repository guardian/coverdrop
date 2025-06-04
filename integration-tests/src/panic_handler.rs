use lazy_static::lazy_static;
use std::panic::PanicHookInfo;
use std::{
    ffi::OsString,
    ops::DerefMut,
    os::unix::prelude::OsStringExt,
    panic,
    process::Command,
    sync::{Mutex, Once},
};

// Panic hooks are how Rust deals with panics
type PanicHook = Box<dyn Fn(&PanicHookInfo<'_>) + Send + Sync + 'static>;

// Container panic hooks are our simplified panic handlers which are called
// when a test fails. They don't need access to the panic info since they're just
// dumping the docker logs.
type ContainerPanicHook = Box<dyn Fn() + Send + Sync + 'static>;

lazy_static! {
    static ref PANIC_HANDLERS: Mutex<Vec<ContainerPanicHook>> = Mutex::new(vec![]);
    static ref DEFAULT_PANIC_HANDLER: Mutex<Option<PanicHook>> = Mutex::new(None);
}

static SETUP: Once = Once::new();

pub fn register_container_panic_hook(name: &'static str, id: &str) {
    let id = id.to_owned();
    let hook = Box::new(move || {
        let output = Command::new("docker")
            .args(["logs", &id])
            .output()
            .expect("execute process");

        println!("Docker logs: {name}");
        println!("STDOUT");
        println!("------------------------");
        println!("{}", OsString::from_vec(output.stdout).to_string_lossy());
        println!("STDERR");
        println!("------------------------");
        println!("{}", OsString::from_vec(output.stderr).to_string_lossy());
        println!("=======================================================");
    });
    PANIC_HANDLERS.lock().unwrap().push(hook);

    SETUP.call_once(|| {
        let prev = panic::take_hook();
        let mut default_panic_handler = DEFAULT_PANIC_HANDLER.lock().unwrap();
        *default_panic_handler.deref_mut() = Some(prev);

        panic::set_hook(Box::new(|payload| {
            let guard = PANIC_HANDLERS.lock().unwrap();

            for hook in guard.iter() {
                hook();
            }

            if let Some(ref prev) = DEFAULT_PANIC_HANDLER.lock().unwrap().as_ref() {
                prev(payload);
            }
        }));
    });
}
