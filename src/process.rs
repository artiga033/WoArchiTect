use snafu::{ResultExt, Snafu};
use windows::Win32::{
    Foundation::{ERROR_NO_MORE_FILES, HANDLE, WIN32_ERROR},
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    },
};

pub struct Process {
    pub pid: u32,
    pub exe_path: String,
}
impl From<PROCESSENTRY32W> for Process {
    fn from(pe32w: PROCESSENTRY32W) -> Self {
        let pid = pe32w.th32ProcessID;
        let exe_path = String::from_utf16_lossy({
            let end = pe32w
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(pe32w.szExeFile.len());
            &pe32w.szExeFile[..end]
        });
        Process { pid, exe_path }
    }
}

pub fn enumrate_running_processes() -> Result<impl Iterator<Item = Result<Process>>> {
    let process_snap =
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).context(WindowsSnafu)? };
    struct Iter {
        process_snap: HANDLE,
        pe32w: PROCESSENTRY32W,
    }
    impl Iterator for Iter {
        type Item = Result<Process>;

        fn next(&mut self) -> Option<Self::Item> {
            // if dwSize is not set, this is the first call
            if self.pe32w.dwSize == 0 {
                self.pe32w.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
                unsafe {
                    if let Err(e) =
                        Process32FirstW(self.process_snap, &mut self.pe32w).context(WindowsSnafu)
                    {
                        return Some(Err(e));
                    }
                }
            } else {
                unsafe {
                    if let Err(e) = Process32NextW(self.process_snap, &mut self.pe32w) {
                        if let Some(ERROR_NO_MORE_FILES) = WIN32_ERROR::from_error(&e) {
                            return None;
                        }
                    }
                }
            }
            Some(Ok(self.pe32w.into()))
        }
    }
    Ok(Iter {
        process_snap,
        pe32w: PROCESSENTRY32W::default(),
    })
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("windows api error: {}", source))]
    Windows { source: windows::core::Error },
}
type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumrate_running_processes() {
        let processes = enumrate_running_processes().unwrap();
        for process in processes {
            let process = process.unwrap();
            println!(
                "Process ID: {}, Executable: {}",
                process.pid, process.exe_path
            );
        }
    }
}
