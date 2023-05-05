pub mod api;

use rhai_dylib::rhai::{
   config::hashing::set_ahash_seed, exported_module, ImmutableString, Module, Shared
};

/// Export the wellness_plugin module.
#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn module_entrypoint() -> Shared<Module> {
    set_ahash_seed(Some([1, 2, 3, 4])).unwrap();

    println!("plugin: {:?}", std::any::TypeId::of::<ImmutableString>());

    exported_module!(api::wellness_plugin).into()
}
