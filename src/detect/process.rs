use super::error::*;
use snafu::{OptionExt, ResultExt};
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::{
        SystemInformation::{IMAGE_FILE_MACHINE, IMAGE_FILE_MACHINE_UNKNOWN},
        Threading::{IsWow64Process2, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    },
};

use crate::architecture;

pub fn detect_process_architecture(h_process: HANDLE) -> Result<architecture::Architecture> {
    let image_file_machine = unsafe {
        let mut process_machine = IMAGE_FILE_MACHINE::default();
        let mut native_machine = IMAGE_FILE_MACHINE::default();
        IsWow64Process2(h_process, &mut process_machine, Some(&mut native_machine)).context(
            WindowsDetailedSnafu {
                call: "IsWow64Process2".to_string(),
                op: "detect_process_architecture".to_string(),
            },
        )?;
        if process_machine != IMAGE_FILE_MACHINE_UNKNOWN {
            process_machine
        } else if native_machine == IMAGE_FILE_MACHINE_UNKNOWN {
            InvalidImageFileMachineSnafu {
                machine: IMAGE_FILE_MACHINE_UNKNOWN.0,
            }
            .fail()?
        } else {
            native_machine
        }
    };
    image_file_machine
        .0
        .try_into()
        .ok()
        .context(InvalidImageFileMachineSnafu {
            machine: image_file_machine.0,
        })
}

pub fn detect_executable_architecture_by_pid(pid: u32) -> Result<architecture::Architecture> {
    let h_process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.context(
        WindowsDetailedSnafu {
            call: "OpenProcess".to_string(),
            op: format!("detect pid {pid}"),
        },
    )?;
    let arch = detect_process_architecture(h_process)?;
    unsafe { CloseHandle(h_process) }.context(WindowsDetailedSnafu {
        call: "CloseHandle".to_string(),
        op: "",
    })?;
    Ok(arch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::{GetCurrentProcess, GetCurrentProcessId};

    #[test]
    fn test_detect_process_architecture() {
        let h_process = unsafe { GetCurrentProcess() };
        let arch = detect_process_architecture(h_process).unwrap();
        println!("Detected architecture: {arch:?}");
    }

    #[test]
    fn test_detect_executable_architecture_by_pid() {
        let pid = unsafe { GetCurrentProcessId() };
        let arch = detect_executable_architecture_by_pid(pid).unwrap();
        println!("Detected architecture by PID: {arch:?}");
    }
}
