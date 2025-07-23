//! Detect architecture of an executable by its PE header.

use std::io::{Read, Seek, SeekFrom};

use super::error::*;
use object::coff::CoffHeader;
use snafu::{OptionExt, ResultExt};

use crate::architecture::Architecture;

pub fn detect_executable_architecture<R>(mut bytes: R) -> Result<Architecture>
where
    R: Read + Seek,
{
    let mut buf = [0; 64];
    bytes.read_exact(&mut buf)?;

    let dos = object::pe::ImageDosHeader::parse(buf.as_slice()).context(ObjectSnafu)?;

    let e_lfanew = dos.nt_headers_offset() as u64;
    let seek_offset = e_lfanew + 4; // the pe signature "PE\0\0"
    bytes.seek(SeekFrom::Start(seek_offset))?;
    let buf = &mut buf[0..20];
    bytes.read_exact(buf)?;

    let mut offset = 0;
    let coff_header =
        object::pe::ImageFileHeader::parse(&*buf, &mut offset).context(ObjectSnafu)?;
    let machine = coff_header.machine();
    machine
        .try_into()
        .ok()
        .context(InvalidImageFileMachineSnafu { machine })
}

pub fn detect_executable_architecture_file<P>(path: P) -> Result<Architecture>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(path)?;
    detect_executable_architecture(file)
}

#[cfg(test)]
mod test {

    use super::*;

    const PE_X86: &[u8] = include_bytes!("../../test_assets/testbin_i686-pc-windows-msvc.exe");
    const PE_X64: &[u8] = include_bytes!("../../test_assets/testbin_x86_64-pc-windows-msvc.exe");
    const PE_ARM64: &[u8] = include_bytes!("../../test_assets/testbin_aarch64-pc-windows-msvc.exe");
    const PE_ARM64EC: &[u8] =
        include_bytes!("../../test_assets/testbin_arm64ec-pc-windows-msvc.exe");

    #[test]
    fn test_detect_executable_architecture() {
        let bins = [PE_X86, PE_X64, PE_ARM64, PE_ARM64EC];
        let expected_architectures = [
            Architecture::I386,
            Architecture::Amd64,
            Architecture::Arm64,
            Architecture::Amd64,
        ];
        for (bin, expected_arch) in bins.into_iter().zip(expected_architectures.into_iter()) {
            let arch = detect_executable_architecture(std::io::Cursor::new(bin))
                .expect("Failed to detect architecture");
            assert_eq!(arch, expected_arch, "Architecture mismatch for binary");
        }
    }
}
