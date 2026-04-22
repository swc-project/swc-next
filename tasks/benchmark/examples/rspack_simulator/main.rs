use std::env;

pub mod experimental;
pub mod legacy;
pub mod oxc;

#[global_allocator]
#[cfg(not(feature = "tracy"))]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[global_allocator]
#[cfg(feature = "tracy")]
static GLOBAL: tracy_client::ProfiledAllocator<std::alloc::System> =
    tracy_client::ProfiledAllocator::new(std::alloc::System, 10);

pub fn main() {
    #[cfg(feature = "tracy")]
    {
        use tracy_client::register_demangler;
        tracy_client::Client::start();
        register_demangler!();
    }

    let source = include_str!("../../files/typescript.js");
    let target = env::var("TARGET").unwrap();
    if target == "legacy" {
        legacy::run(source);
    }

    if target == "exp" {
        experimental::run(source, None);
    }

    if target == "exp_compat" {
        experimental::run(source, Some(experimental::CompatMode::Safe));
    }

    if target == "exp_compat_safe" {
        experimental::run(source, Some(experimental::CompatMode::Safe));
    }

    if target == "exp_compat_unsafe" {
        experimental::run(source, Some(experimental::CompatMode::Unsafe));
    }

    if target == "oxc" {
        oxc::run(source);
    }

    #[cfg(feature = "tracy")]
    std::thread::sleep(std::time::Duration::from_secs(2));
}
