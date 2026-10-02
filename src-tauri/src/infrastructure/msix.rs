//! Detects whether this process is running from an installed MSIX package
//! (a Microsoft Store install) rather than launched directly as an
//! unpackaged EXE.

/// True when the current process has an MSIX package identity - i.e. it was
/// installed via the Microsoft Store (or `Add-AppxPackage`), not run as a
/// plain unpackaged .exe. MSIX installs get updates through Windows Update
/// automatically, so callers use this to skip update-check logic that only
/// applies to the unpackaged NSIS distribution.
#[cfg(target_os = "windows")]
pub fn is_msix_package() -> bool {
    use windows_sys::Win32::Foundation::APPMODEL_ERROR_NO_PACKAGE;
    use windows_sys::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;

    let mut length: u32 = 0;
    // Probing with a null buffer just returns the required length (or
    // APPMODEL_ERROR_NO_PACKAGE if there's no package identity at all) - we
    // only care which of those happened, not the actual package name.
    let result = unsafe { GetCurrentPackageFullName(&mut length, std::ptr::null_mut()) };
    result != APPMODEL_ERROR_NO_PACKAGE
}

#[cfg(not(target_os = "windows"))]
pub fn is_msix_package() -> bool {
    false
}

#[cfg(all(test, not(target_os = "windows")))]
mod tests {
    use super::*;

    #[test]
    fn non_windows_builds_report_no_msix_package() {
        assert!(!is_msix_package());
    }
}
