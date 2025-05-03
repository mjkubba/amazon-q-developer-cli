use std::sync::{
    Arc,
    Mutex,
};
use std::ffi::OsString;

use sysinfo::System;
use crate::Shim;

#[derive(Debug, Clone, Default)]
pub struct SysInfo(inner::Inner);

mod inner {
    use std::collections::HashSet;
    use std::sync::{
        Arc,
        Mutex,
    };

    #[derive(Debug, Clone, Default)]
    pub enum Inner {
        #[default]
        Real,
        Fake(Arc<Mutex<Fake>>),
    }

    #[derive(Debug, Clone, Default)]
    pub struct Fake {
        pub process_names: HashSet<String>,
    }
}

impl SysInfo {
    pub fn new_fake() -> Self {
        Self(inner::Inner::Fake(Arc::new(Mutex::new(inner::Fake::default()))))
    }

    /// Returns whether the process containing `name` is running.
    pub fn is_process_running(&self, name: &str) -> bool {
        use inner::Inner;
        match &self.0 {
            Inner::Real => {
                let mut system = System::new_all();
                system.refresh_all();
                let name_os = OsString::from(name);
                let is_running = system.processes_by_name(&name_os).next().is_some();
                is_running
            },
            Inner::Fake(fake) => fake.lock().unwrap().process_names.contains(name),
        }
    }

    pub fn add_running_processes(&self, process_names: &[&str]) {
        use inner::Inner;
        match &self.0 {
            Inner::Real => panic!("unimplemented"),
            Inner::Fake(fake) => {
                let curr_names = &mut fake.lock().unwrap().process_names;
                for name in process_names {
                    curr_names.insert((*name).to_string());
                }
            },
        }
    }
}

impl Shim for SysInfo {
    fn is_real(&self) -> bool {
        matches!(self.0, inner::Inner::Real)
    }
}
