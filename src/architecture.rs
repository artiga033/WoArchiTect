use strum::FromRepr;

#[derive(Debug, Clone, Copy, PartialEq, FromRepr)]
#[cfg_attr(test, derive(strum::EnumIter))]
#[repr(u16)]
pub enum Architecture {
    I386 = 0x014c,
    R3000 = 0x0162,
    R4000 = 0x0166,
    R10000 = 0x0168,
    WceMipsV2 = 0x0169,
    Alpha = 0x0184,
    Sh3 = 0x01a2,
    Sh3Dsp = 0x01a3,
    Sh3E = 0x01a4,
    Sh4 = 0x01a6,
    Sh5 = 0x01a8,
    Arm = 0x01c0,
    Thumb = 0x01c2,
    ArmNt = 0x01c4,
    Am33 = 0x01d3,
    PowerPc = 0x01f0,
    PowerPcFp = 0x01f1,
    Ia64 = 0x0200,
    Mips16 = 0x0266,
    #[allow(non_camel_case_types)]
    Alpha64_Axp64 = 0x0284,
    MipsFpu = 0x0366,
    MipsFpu16 = 0x0466,
    Tricore = 0x0520,
    Cef = 0x0cef,
    Ebc = 0x0ebc,
    Amd64 = 0x8664,
    M32R = 0x9041,
    Arm64 = 0xaa64,
    Cee = 0xc0ee,
}

impl TryFrom<windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE> for Architecture {
    type Error = ();
    fn try_from(
        value: windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE,
    ) -> Result<Self, Self::Error> {
        Self::from_repr(value.0).ok_or(())
    }
}
impl From<Architecture> for windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE {
    fn from(val: Architecture) -> Self {
        windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE(val as u16)
    }
}
impl TryFrom<u16> for Architecture {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::from_repr(value).ok_or(())
    }
}
impl From<Architecture> for u16 {
    fn from(val: Architecture) -> Self {
        val as u16
    }
}
impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Windows Task Manager style naming, fallback to debug format now if we don't know how Task Manager would display it...
        match self {
            Architecture::I386 => f.write_str("x86"),
            Architecture::Amd64 => f.write_str("x64"),
            Architecture::Arm64 => f.write_str("ARM64"),
            _ => f.write_fmt(format_args!("{self:?}")),
        }
    }
}

impl Architecture {
    pub fn current() -> Self {
        crate::detect::current::get_current_sys_architecture()
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn from_and_to_windows() {
        let mut theirs = [
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_ALPHA,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_ALPHA64,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_AM33,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_AMD64,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_ARM,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_ARM64,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_ARMNT,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_AXP64,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_CEE,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_CEF,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_EBC,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_I386,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_IA64,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_M32R,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_MIPS16,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_MIPSFPU,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_MIPSFPU16,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_POWERPC,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_POWERPCFP,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_R10000,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_R3000,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_R4000,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_SH3,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_SH3DSP,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_SH3E,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_SH4,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_SH5,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_TARGET_HOST,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_THUMB,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_TRICORE,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_UNKNOWN,
            windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_WCEMIPSV2,
        ];
        theirs.sort_by_key(|x| x.0);
        let mut ours = Architecture::iter()
            // Alpha64 and AXP64 are of the same value on our side, so we repeat them to keep the order
            .chain([Architecture::Alpha64_Axp64])
            .collect::<Vec<_>>();
        ours.sort_by_key(|x| *x as u16);

        // the first two are Unknown and TargetHost, which we don't support and expects an error
        let ours = [None, None]
            .into_iter()
            .chain(ours.into_iter().map(Some))
            .collect::<Vec<_>>();
        for (their, our) in theirs.iter().zip(ours.iter()) {
            let our_from_their: Result<Architecture, ()> = (*their).try_into();
            if let Some(our) = our {
                assert_eq!(our_from_their.unwrap(), *our);
            } else {
                assert!(our_from_their.is_err());
            }
        }
        for our in ours.iter().filter_map(|x| x.as_ref().map(|x| *x)) {
            let their: windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE = our.into();
            assert_eq!(their.0, our.into());
        }
    }
}
