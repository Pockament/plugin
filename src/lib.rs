use std::sync::Arc;

use rune::runtime::GuardedArgs;
pub use rune::Value;
use rune::{Context, Source, Sources, Vm};

mod http_server;

static mut RUNE: Option<Box<Vm>> = None;

pub fn initialized() -> bool { unsafe { RUNE.is_some() } }

pub fn init(src_codes: &[(&str, &str)]) -> Result<(), String> {
    let mut ctx = Context::new();
    ctx.install(&http_server::make_module()).unwrap();
    let ctx_arc = Arc::new(ctx.runtime());

    let mut srcs = Sources::new();
    src_codes
        .iter()
        .map(|(n, s)| Source::new(n, s))
        .map(|src| srcs.insert(src))
        .count();

    let unit = rune::prepare(&mut srcs)
        .build()
        .map_err(|e| e.to_string())?;
    let unit_arc = Arc::new(unit);

    let vm = Vm::new(ctx_arc, unit_arc);
    let vm_box = Box::new(vm);

    unsafe {
        RUNE = Some(vm_box);
    }

    Ok(())
}

pub fn uninit() {
    unsafe {
        RUNE = None;
    }
}

#[allow(clippy::result_unit_err)]
pub fn run<A: GuardedArgs>(name: &[&str], args: A) -> Result<Result<Value, String>, ()> {
    if unsafe { RUNE.is_none() } {
        return Err(());
    }

    let result = unsafe { RUNE.as_mut() }
        .unwrap()
        .call(name, args)
        .map_err(|e| e.to_string());

    Ok(result)
}

mod test {
    #[test]
    fn no_init() { assert!(!super::initialized()) }

    #[test]
    fn init() {
        assert!(!super::initialized());

        let result = super::init(&[("script", "")]);
        assert!(result.is_ok());
        assert!(super::initialized());
    }

    #[test]
    fn execute() {
        assert!(!super::initialized());

        let src = r#"
        pub fn add(l, r) {
            l + r
        }

        pub fn echo(v) {
            v
        }

        pub fn inc(v) {
            v + 1
        }
            "#;

        let result = super::init(&[("script", src)]);
        assert!(result.is_ok());
        assert!(super::initialized());

        let r = super::run(&["add"], (3i32, 5i32));
        assert_eq!(r.unwrap().unwrap().into_integer().unwrap(), 8);

        let r = super::run(&["echo"], ((),));
        r.unwrap().unwrap().into_unit().unwrap();

        let r = super::run(&["inc"], (9i32,));
        assert_eq!(r.unwrap().unwrap().into_integer().unwrap(), 10);
    }
}
