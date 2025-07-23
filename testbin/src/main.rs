#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::panic::PanicInfo;

use windows_sys::Win32::System::Console::GetStdHandle;
use windows_sys::Win32::System::Console::STD_OUTPUT_HANDLE;
use windows_sys::Win32::System::Console::WriteConsoleA;
use windows_sys::Win32::System::Threading::ExitProcess;


#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    unsafe {
        ExitProcess(1);
    }
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
fn mainCRTStartup() -> ! {
    let message = "hello world\n";
    unsafe {
        let console = GetStdHandle(STD_OUTPUT_HANDLE);

        WriteConsoleA(
            console,
            message.as_ptr().cast::<u8>(),
            message.len() as u32,
            core::ptr::null_mut(),
            core::ptr::null(),
        );

        ExitProcess(0)
    }
}
