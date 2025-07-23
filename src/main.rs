use std::sync::LazyLock;

use comfy_table::Table;
use palc::Parser;
use snafu::Snafu;

use crate::architecture::Architecture;

mod architecture;
mod detect;
mod executable;
mod process;

static ARGS: LazyLock<Args> = LazyLock::new(Args::parse);
fn main() -> Result<()> {
    if !ARGS.no_processes {
        println!("current running processes:\n{}", detect_processes()?);
    }
    if !ARGS.no_executables {
        println!("executables found in PATH:\n{}", detect_executables()?);
    }
    Ok(())
}

fn detect_processes() -> Result<Table> {
    let mut table = Table::new();
    table.set_header(vec![
        "PID".to_string(),
        "Executable".to_string(),
        "Architecture".to_string(),
    ]);
    let processes = process::enumrate_running_processes()?;
    for process in processes {
        let process = process?;
        match detect::process::detect_executable_architecture_by_pid(process.pid) {
            Ok(arch) => {
                if !ARGS.all && arch == Architecture::current() {
                    continue;
                }
                table.add_row(vec![
                    process.pid.to_string(),
                    process.exe_path,
                    arch.to_string(),
                ]);
            }
            Err(_) => {
                // eprintln!("failed detect {}({}), {}", process.exe_path, process.pid, e);
            }
        }
    }
    Ok(table)
}

fn detect_executables() -> Result<Table> {
    let mut table = Table::new();
    table.set_header(vec!["Executable".to_string(), "Architecture".to_string()]);

    let executables = executable::enumrate_executables()?;
    for exe_path in executables {
        if let Ok(arch) = detect::pe::detect_executable_architecture_file(&exe_path) {
            if !ARGS.all && arch == Architecture::current() {
                continue;
            }
            table.add_row(vec![exe_path.display().to_string(), arch.to_string()]);
        }
    }

    Ok(table)
}

#[derive(Debug, palc::Parser)]
struct Args {
    /// Do not detect current running processes
    #[arg(short = 'P', long)]
    no_processes: bool,
    /// Do not detect all available executables
    ///
    /// Executables are PE files that can be executed on the system.
    /// They are located by enumrating all files in the `PATH` environment variable.
    #[arg(short = 'E', long)]
    no_executables: bool,
    /// show all results, by default only result that are not same with current machines architecture are shown
    #[arg(short, long)]
    all: bool,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(context(false))]
    #[snafu(display("when detecting architecture, {}", source))]
    Detect { source: detect::Error },
    #[snafu(context(false))]
    #[snafu(display("when enumrating processes, {}", source))]
    EnumrateProcesses { source: process::Error },
    #[snafu(context(false))]
    #[snafu(display("when enumrating executables, {}", source))]
    EnumrateExecutables { source: executable::Error },
}
type Result<T> = std::result::Result<T, Error>;
