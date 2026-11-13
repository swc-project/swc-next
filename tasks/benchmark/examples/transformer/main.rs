use std::env;

mod next;
mod oxc;
mod swc;

#[global_allocator]
#[cfg(not(feature = "tracy"))]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[global_allocator]
#[cfg(feature = "tracy")]
static GLOBAL: tracy_client::ProfiledAllocator<std::alloc::System> =
    tracy_client::ProfiledAllocator::new(std::alloc::System, 10);

fn main() {
    #[cfg(feature = "tracy")]
    {
        use tracy_client::register_demangler;
        tracy_client::Client::start();
        register_demangler!();
    }

    let source = include_str!("../../files/App.tsx");
    match env::var("TARGET").as_deref() {
        Ok("next") => next::run(source),
        Ok("oxc") => oxc::run(source),
        Ok("swc") => swc::run(source),
        _ => panic!("set TARGET to next, oxc, or swc"),
    }

    #[cfg(feature = "tracy")]
    std::thread::sleep(std::time::Duration::from_secs(2));
}
