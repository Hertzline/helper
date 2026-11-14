//! Create a Windows System Restore point (HZ-019 / AC-7.4).
//!
//! Official path (Microsoft Learn):
//! - LoadLibrary SrClient.dll + SRSetRestorePointW (do not link at load time)
//! - BEGIN_SYSTEM_CHANGE then END_SYSTEM_CHANGE
//! - MODIFY_SETTINGS restore point type
//!
//! Sources:
//! - https://learn.microsoft.com/en-us/windows/win32/api/srrestoreptapi/nf-srrestoreptapi-srsetrestorepointw
//! - https://learn.microsoft.com/en-us/windows/win32/sr/using-system-restore
//!
//! Linux / macOS: not possible (ARCHITECTURE = N). Tests never require a live
//! restore point; the Windows path reports honest failure if SR is disabled.

use serde::{Deserialize, Serialize};

/// Default description when the caller omits one.
pub const DEFAULT_DESCRIPTION: &str = "Hertzline before system change";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateRestorePointReport {
    pub os: String,
    pub status: String,
    pub description: String,
    pub sequence_number: Option<i64>,
    pub detail: String,
    pub source_url: String,
}

/// Allow-listed helper op: create one System Restore point when possible.
pub fn create_restore_point(description: Option<&str>) -> CreateRestorePointReport {
    let description = description
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_DESCRIPTION)
        .to_string();

    #[cfg(windows)]
    {
        windows_impl::create(&description)
    }
    #[cfg(not(windows))]
    {
        CreateRestorePointReport {
            os: std::env::consts::OS.into(),
            status: "not_possible".into(),
            description,
            sequence_number: None,
            detail: "System Restore points are Windows-only (AC-7.4 / ARCHITECTURE = N)".into(),
            source_url: "https://learn.microsoft.com/en-us/windows/win32/sr/using-system-restore"
                .into(),
        }
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    const BEGIN_SYSTEM_CHANGE: u32 = 100;
    const END_SYSTEM_CHANGE: u32 = 101;
    const MODIFY_SETTINGS: u32 = 12;
    const MAX_DESC_W: usize = 256;
    const COINIT_APARTMENTTHREADED: u32 = 0x2;
    const ERROR_SERVICE_DISABLED: u32 = 1058;
    const SOURCE: &str = "https://learn.microsoft.com/en-us/windows/win32/sr/using-system-restore";

    #[repr(C, packed(1))]
    struct RestorePointInfoW {
        dw_event_type: u32,
        dw_restore_pt_type: u32,
        ll_sequence_number: i64,
        sz_description: [u16; MAX_DESC_W],
    }

    #[repr(C, packed(1))]
    struct StateMgrStatus {
        n_status: u32,
        ll_sequence_number: i64,
    }

    type SrSetRestorePointW =
        unsafe extern "system" fn(*const RestorePointInfoW, *mut StateMgrStatus) -> i32;

    #[link(name = "ole32")]
    extern "system" {
        fn CoInitializeEx(pvreserved: *mut c_void, dwcoinit: u32) -> i32;
        fn CoUninitialize();
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryW(name: *const u16) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
        fn FreeLibrary(module: *mut c_void) -> i32;
    }

    pub fn create(description: &str) -> CreateRestorePointReport {
        // COM init required before SRSetRestorePoint (Microsoft Using System Restore).
        let hr = unsafe { CoInitializeEx(ptr::null_mut(), COINIT_APARTMENTTHREADED) };
        let com_ok = hr == 0 || hr == 1; // S_OK or S_FALSE
        if !com_ok && hr != 0x80010106u32 as i32 {
            // RPC_E_CHANGED_MODE: already initialized differently; continue.
            // Other failures: still attempt SrClient (honest status if it fails).
        }

        let result = create_inner(description);
        if com_ok {
            unsafe { CoUninitialize() };
        }
        result
    }

    fn create_inner(description: &str) -> CreateRestorePointReport {
        let dll_name: Vec<u16> = std::ffi::OsStr::new("SrClient.dll")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let module = unsafe { LoadLibraryW(dll_name.as_ptr()) };
        if module.is_null() {
            return CreateRestorePointReport {
                os: "windows".into(),
                status: "unavailable".into(),
                description: description.into(),
                sequence_number: None,
                detail: "LoadLibraryW SrClient.dll failed".into(),
                source_url: SOURCE.into(),
            };
        }

        let proc = unsafe { GetProcAddress(module, c"SRSetRestorePointW".as_ptr().cast()) };
        if proc.is_null() {
            unsafe { FreeLibrary(module) };
            return CreateRestorePointReport {
                os: "windows".into(),
                status: "unavailable".into(),
                description: description.into(),
                sequence_number: None,
                detail: "GetProcAddress SRSetRestorePointW failed".into(),
                source_url: SOURCE.into(),
            };
        }
        let set_rp: SrSetRestorePointW = unsafe { std::mem::transmute(proc) };

        let mut begin = RestorePointInfoW {
            dw_event_type: BEGIN_SYSTEM_CHANGE,
            dw_restore_pt_type: MODIFY_SETTINGS,
            ll_sequence_number: 0,
            sz_description: [0; MAX_DESC_W],
        };
        write_desc_packed(&mut begin, description);

        let mut status = StateMgrStatus {
            n_status: 0,
            ll_sequence_number: 0,
        };
        let ok_begin = unsafe { set_rp(&begin, &mut status) };
        // packed(1): copy fields before use to avoid unaligned references
        let begin_n_status = unsafe { ptr::addr_of!(status.n_status).read_unaligned() };
        let sequence = unsafe { ptr::addr_of!(status.ll_sequence_number).read_unaligned() };
        if ok_begin == 0 {
            unsafe { FreeLibrary(module) };
            let detail = if begin_n_status == ERROR_SERVICE_DISABLED {
                "System Restore is turned off".into()
            } else {
                format!("SRSetRestorePointW BEGIN failed; nStatus={begin_n_status}")
            };
            return CreateRestorePointReport {
                os: "windows".into(),
                status: "failed".into(),
                description: description.into(),
                sequence_number: None,
                detail,
                source_url: SOURCE.into(),
            };
        }

        let mut end = RestorePointInfoW {
            dw_event_type: END_SYSTEM_CHANGE,
            dw_restore_pt_type: MODIFY_SETTINGS,
            ll_sequence_number: sequence,
            sz_description: [0; MAX_DESC_W],
        };
        write_desc_packed(&mut end, description);
        let mut end_status = StateMgrStatus {
            n_status: 0,
            ll_sequence_number: 0,
        };
        let ok_end = unsafe { set_rp(&end, &mut end_status) };
        let end_n_status = unsafe { ptr::addr_of!(end_status.n_status).read_unaligned() };
        unsafe { FreeLibrary(module) };

        if ok_end == 0 {
            return CreateRestorePointReport {
                os: "windows".into(),
                status: "partial".into(),
                description: description.into(),
                sequence_number: Some(sequence),
                detail: format!(
                    "BEGIN succeeded (seq={sequence}) but END failed; nStatus={end_n_status}"
                ),
                source_url: SOURCE.into(),
            };
        }

        CreateRestorePointReport {
            os: "windows".into(),
            status: "ok".into(),
            description: description.into(),
            sequence_number: Some(sequence),
            detail: format!("System Restore point created (sequence {sequence})"),
            source_url: SOURCE.into(),
        }
    }

    fn write_desc_packed(info: &mut RestorePointInfoW, text: &str) {
        let mut local = [0u16; MAX_DESC_W];
        let wide: Vec<u16> = text.encode_utf16().collect();
        let max = MAX_DESC_W.saturating_sub(1);
        let len = wide.len().min(max);
        local[..len].copy_from_slice(&wide[..len]);
        unsafe {
            ptr::copy_nonoverlapping(
                local.as_ptr(),
                ptr::addr_of_mut!(info.sz_description).cast::<u16>(),
                MAX_DESC_W,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_windows_reports_not_possible() {
        let report = create_restore_point(Some("test"));
        #[cfg(windows)]
        {
            assert_eq!(report.os, "windows");
            assert!(
                report.status == "ok"
                    || report.status == "failed"
                    || report.status == "partial"
                    || report.status == "unavailable",
                "unexpected status: {} ({})",
                report.status,
                report.detail
            );
            assert!(!report.source_url.is_empty());
        }
        #[cfg(not(windows))]
        {
            assert_eq!(report.status, "not_possible");
            assert!(report.sequence_number.is_none());
        }
    }

    #[test]
    fn default_description_used_when_empty() {
        let report = create_restore_point(Some("  "));
        assert_eq!(report.description, DEFAULT_DESCRIPTION);
    }
}
