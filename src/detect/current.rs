use std::sync::LazyLock;

use snafu::{OptionExt, ResultExt};
use windows::Win32::{
    Foundation::HANDLE,
    System::{
        SystemInformation::IMAGE_FILE_MACHINE,
        Threading::{GetCurrentProcess, IsWow64Process2},
    },
};

use super::error::*;
use crate::architecture::Architecture;

static CURRENT_ARCHITECTURE_CACHE: LazyLock<Architecture> = LazyLock::new(|| {
    || -> Result<Architecture> {
        let mut native_machine = IMAGE_FILE_MACHINE::default();
        unsafe {
            let current_process: HANDLE = GetCurrentProcess();
            let mut process_machine = IMAGE_FILE_MACHINE::default();
            IsWow64Process2(
                current_process,
                &mut process_machine,
                Some(&mut native_machine),
            )
            .context(WindowsDetailedSnafu {
                op: "get current system architecture",
                call: "IsWow64Process2",
            })?;
        }
        native_machine
            .try_into()
            .ok()
            .context(InvalidImageFileMachineSnafu {
                machine: native_machine.0,
            })
    }()
    .expect("Failed to get current system architecture")
});

pub fn get_current_sys_architecture() -> Architecture {
    *CURRENT_ARCHITECTURE_CACHE
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_current_sys_architecture() {
        let arch = get_current_sys_architecture();
        println!("Current system architecture: {arch:?}");
    }
}
