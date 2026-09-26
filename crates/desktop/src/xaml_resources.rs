use std::ffi::c_void;
use std::ptr;
use std::sync::Once;

use windows_collections::IMap;
use windows_core::{HRESULT, HSTRING, IInspectable, Interface, Result, RuntimeName};
use windows_reference::IReference;

const TRANSPARENT_KEYS: [&str; 2] = [
    "NavigationViewContentBackground",
    "NavigationViewContentGridBorderBrush",
];

const NAV_ICON_BOX_KEY: &str = "NavigationViewItemOnLeftIconBoxHeight";

#[allow(non_camel_case_types, non_snake_case)]
#[repr(C)]
pub struct IApplicationStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Current: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
}

windows_core::imp::define_interface!(
    IApplicationStatics,
    IApplicationStatics_Vtbl,
    0x4e0d09f5_4358_512c_a987_503b52848e95
);

#[allow(non_camel_case_types, non_snake_case)]
#[repr(C)]
pub struct IApplication_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Resources: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
}

windows_core::imp::define_interface!(
    IApplication,
    IApplication_Vtbl,
    0x06a8f4e7_1146_55af_820d_ebd55643b021
);

struct XamlApplication;

impl RuntimeName for XamlApplication {
    const NAME: &'static str = "Microsoft.UI.Xaml.Application";
}

struct SolidColorBrush;

impl RuntimeName for SolidColorBrush {
    const NAME: &'static str = "Microsoft.UI.Xaml.Media.SolidColorBrush";
}

pub fn override_navigation_view_resources() {
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        if let Err(err) = install() {
            tracing::warn!(%err, "could not override NavigationView resources");
        }
    });
}

fn install() -> Result<()> {
    let statics = windows_core::factory::<XamlApplication, IApplicationStatics>()?;
    let application = unsafe {
        let mut current = ptr::null_mut();
        (Interface::vtable(&statics).Current)(Interface::as_raw(&statics), &mut current).ok()?;
        IInspectable::from_raw(current)
    };

    let application: IApplication = application.cast()?;
    let resources = unsafe {
        let mut resources = ptr::null_mut();
        (Interface::vtable(&application).Resources)(
            Interface::as_raw(&application),
            &mut resources,
        )
        .ok()?;
        IInspectable::from_raw(resources)
    };

    let resources: IMap<IInspectable, IInspectable> = resources.cast()?;
    let transparent =
        windows_core::factory::<SolidColorBrush, windows_core::imp::IGenericFactory>()?
            .ActivateInstance::<IInspectable>()?;
    for name in TRANSPARENT_KEYS {
        let key: IInspectable = IReference::<HSTRING>::from(name).into();
        resources.Insert(&key, &transparent)?;
    }

    let key: IInspectable = IReference::<HSTRING>::from(NAV_ICON_BOX_KEY).into();
    let height: IInspectable = IReference::<f64>::from(ui::theme::size::NavIcon).into();
    resources.Insert(&key, &height)?;

    Ok(())
}
