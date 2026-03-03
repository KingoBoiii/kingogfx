use std::ffi::c_void;

pub trait GLProcLoader {
    fn get_proc_address(&mut self, procname: &str) -> *const c_void;
}