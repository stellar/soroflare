// The content of these two mods is created on compile time (build.rs)
pub mod contracts {
    include!(concat!(env!("OUT_DIR"), "/embedded_contracts.rs"));
}
