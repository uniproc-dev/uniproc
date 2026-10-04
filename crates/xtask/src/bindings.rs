#[inline]
pub unsafe fn FreeLibrary(hlibmodule: HMODULE) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn FreeLibrary(hlibmodule : HMODULE) -> windows_core::BOOL);
    unsafe { FreeLibrary(hlibmodule) }
}
#[inline]
pub unsafe fn LoadLibraryExW<P0>(lplibfilename: P0, hfile: Option<HANDLE>, dwflags: u32) -> HMODULE
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn LoadLibraryExW(lplibfilename : windows_core::PCWSTR, hfile : HANDLE, dwflags : u32) -> HMODULE);
    unsafe {
        LoadLibraryExW(
            lplibfilename.param().abi(),
            hfile.unwrap_or(core::mem::zeroed()) as _,
            dwflags,
        )
    }
}
#[inline]
pub unsafe fn LoadStringW(
    hinstance: Option<HINSTANCE>,
    uid: u32,
    lpbuffer: windows_core::PWSTR,
    cchbuffermax: i32,
) -> i32 {
    windows_core::link!("user32.dll" "system" fn LoadStringW(hinstance : HINSTANCE, uid : u32, lpbuffer : windows_core::PWSTR, cchbuffermax : i32) -> i32);
    unsafe {
        LoadStringW(
            hinstance.unwrap_or(core::mem::zeroed()) as _,
            uid,
            lpbuffer,
            cchbuffermax,
        )
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HANDLE(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HINSTANCE(pub *mut core::ffi::c_void);
pub type HMODULE = HINSTANCE;
pub const LOAD_LIBRARY_AS_DATAFILE: i32 = 2;
pub const LOAD_LIBRARY_AS_IMAGE_RESOURCE: i32 = 32;
