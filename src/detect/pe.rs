//! Detect architecture of an executable by its PE header.

use std::{
    io::{Read, Seek, SeekFrom},
    mem,
};

use super::error::*;
use object::{
    LittleEndian, ReadRef,
    coff::CoffHeader,
    pe::{
        COMIMAGE_FLAGS_32BITREQUIRED, COMIMAGE_FLAGS_ILONLY, ImageCor20Header,
        ImageOptionalHeader32, ImageSectionHeader,
    },
    read::pe::{DataDirectories, ImageOptionalHeader},
};
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
    let file_header =
        object::pe::ImageFileHeader::parse(&*buf, &mut offset).context(ObjectSnafu)?;
    let machine = file_header.machine();
    if machine == object::pe::IMAGE_FILE_MACHINE_I386 {
        // possibly a .NET assembly
        let sz = file_header.size_of_optional_header.get(LittleEndian);
        enum OwnedBuf {
            Stack([u8; 224]),
            Heap(Vec<u8>),
        }
        let mut owned_buf = if sz <= 224 {
            // on stack buffer
            let buf = [0; 224];
            OwnedBuf::Stack(buf)
        } else {
            // heap buffer
            OwnedBuf::Heap(vec![0; sz as usize])
        };
        let full_buf: &mut [u8] = match &mut owned_buf {
            OwnedBuf::Stack(b) => &mut b[..],
            OwnedBuf::Heap(b) => &mut b[..],
        };
        let buf = &mut full_buf[..sz as usize];
        bytes.read_exact(buf)?;
        let optional_header =
            object::read::ReadRef::read::<object::pe::ImageOptionalHeader32>(&*buf, &mut 0)
                .ok()
                .context(EmptySnafu {
                    msg: "read IMAGE_OPTIONAL_HEADER(PE32)",
                })?;
        let data_directories = DataDirectories::parse(
            &buf[std::mem::size_of::<ImageOptionalHeader32>()..],
            optional_header.number_of_rva_and_sizes(),
        )
        .context(ObjectSnafu)?;
        if let Some(com_descriptor) = data_directories.iter().nth(14) {
            let rva = com_descriptor.virtual_address.get(LittleEndian);
            if rva != 0 {
                let va = com_descriptor.virtual_address.get(LittleEndian);
                let number_of_sections = file_header.number_of_sections.get(LittleEndian) as usize;
                let mut read_count = 0;
                let cor20_header = 'o: loop {
                    if read_count >= number_of_sections {
                        break None;
                    }
                    let batch_size = full_buf.len() / mem::size_of::<ImageSectionHeader>();
                    let batch_size = batch_size.min(number_of_sections - read_count);
                    let buf = &mut full_buf[0..batch_size * mem::size_of::<ImageSectionHeader>()];
                    bytes.read_exact(buf)?;
                    for section_buf in buf.chunks_exact(mem::size_of::<ImageSectionHeader>()) {
                        let section = section_buf
                            .read::<ImageSectionHeader>(&mut 0)
                            .ok()
                            .context(EmptySnafu {
                                msg: "read IMAGE_SECTION_HEADER",
                            })?;
                        read_count += 1;
                        if section.contains_rva(va) {
                            let file_offset = section.pointer_to_raw_data.get(LittleEndian)
                                + (va - section.virtual_address.get(LittleEndian));
                            let buf = &mut full_buf[..mem::size_of::<ImageCor20Header>()];
                            bytes.seek(SeekFrom::Start(file_offset.into()))?;
                            bytes.read_exact(buf)?;
                            break 'o buf.read::<ImageCor20Header>(&mut 0).ok();
                        }
                    }
                };
                if let Some(cor20_header) = cor20_header {
                    let flags = cor20_header.flags.get(LittleEndian);
                    // x86
                    if flags & COMIMAGE_FLAGS_32BITREQUIRED != 0 {
                        return Ok(Architecture::I386);
                    }
                    // Any CPU
                    else if flags & COMIMAGE_FLAGS_ILONLY != 0 {
                        return Ok(Architecture::current());
                    }
                }
            }
        };
    }
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
    const PE_DOTNET: &[u8] = include_bytes!("../../test_assets/mscorlib.dll");

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

    #[test]
    fn test_detect_executable_architecture_dotnet() {
        let arch = detect_executable_architecture(std::io::Cursor::new(PE_DOTNET))
            .expect("Failed to detect architecture for .NET assembly");
        assert_eq!(
            arch,
            Architecture::current(),
            ".NET assembly are always considered the same as the current architecture"
        );
    }
}
