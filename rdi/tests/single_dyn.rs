use rdi::inject;
use rdi::injector;
use std::sync::Arc;

pub trait Db {
    fn call(&self);
}

#[inject]
pub fn handler(#[inject] db: Arc<dyn Db>) {
    db.call()
}

#[test]
fn test_ok() {
    let injector = ok_injector();
    let my_handler = injector.inject(handler);

    my_handler()
}

struct OkDb {}

impl Db for OkDb {
    fn call(&self) {}
}

#[injector]
fn ok_injector() {
    fn db() -> Arc<dyn Db> {
        Arc::new(OkDb {})
    }
}

#[test]
#[should_panic(expected = "Success")]
fn test_panic() {
    let injector = panic_injector();
    let my_handler = injector.inject(handler);

    my_handler()
}

#[injector]
fn panic_injector() {
    fn db() -> Arc<dyn Db> {
        Arc::new(PanicDb {})
    }
}

struct PanicDb {}

impl Db for PanicDb {
    fn call(&self) {
        panic!("Success")
    }
}
