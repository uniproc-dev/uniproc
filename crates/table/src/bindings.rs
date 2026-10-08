#[inline]
pub unsafe fn DWriteCreateFactory<T>(factorytype: DWRITE_FACTORY_TYPE) -> windows_core::Result<T>
where
    T: windows_core::Interface,
{
    windows_core::link!("dwrite.dll" "system" fn DWriteCreateFactory(factorytype : DWRITE_FACTORY_TYPE, iid : *const windows_core::GUID, factory : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    let mut result__ = core::ptr::null_mut();
    unsafe {
        DWriteCreateFactory(factorytype, &T::IID, &mut result__)
            .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn SHCreateMemStream(pinit: Option<&[u8]>) -> Option<IStream> {
    windows_core::link!("shlwapi.dll" "system" fn SHCreateMemStream(pinit : *const u8, cbinit : u32) -> Option < IStream >);
    unsafe {
        SHCreateMemStream(
            pinit.map_or(core::ptr::null(), |slice| slice.as_ptr()),
            pinit.map_or(0, |slice| slice.len().try_into().unwrap()),
        )
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Color {
    pub A: u8,
    pub R: u8,
    pub G: u8,
    pub B: u8,
}
impl windows_core::imp::TypeKind for Color {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Color {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.UI.Color;u1;u1;u1;u1)");
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Color");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionAnimation, CompositionObject);
impl CompositionAnimation {
    pub fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetReferenceParameter)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetScalarParameter)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<Self>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionAnimation>();
}
unsafe impl windows_core::Interface for CompositionAnimation {
    type Vtable = <ICompositionAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionAnimation {
    const NAME: &'static str = "Microsoft.UI.Composition.CompositionAnimation";
}
unsafe impl Send for CompositionAnimation {}
unsafe impl Sync for CompositionAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionBrush(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionBrush,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionBrush, CompositionObject);
impl CompositionBrush {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionBrush>();
}
unsafe impl windows_core::Interface for CompositionBrush {
    type Vtable = <ICompositionBrush as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionBrush as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionBrush {
    const NAME: &'static str = "Microsoft.UI.Composition.CompositionBrush";
}
unsafe impl Send for CompositionBrush {}
unsafe impl Sync for CompositionBrush {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionClip(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionClip,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionClip, CompositionObject);
impl CompositionClip {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionClip {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionClip>();
}
unsafe impl windows_core::Interface for CompositionClip {
    type Vtable = <ICompositionClip as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionClip as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionClip {
    const NAME: &'static str = "Microsoft.UI.Composition.CompositionClip";
}
unsafe impl Send for CompositionClip {}
unsafe impl Sync for CompositionClip {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionColorBrush(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionColorBrush,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionColorBrush, CompositionBrush, CompositionObject);
impl CompositionColorBrush {
    pub fn SetColor(&self, value: Color) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetColor)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionColorBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionColorBrush>();
}
unsafe impl windows_core::Interface for CompositionColorBrush {
    type Vtable = <ICompositionColorBrush as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionColorBrush as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionColorBrush {
    const NAME: &'static str = "Microsoft.UI.Composition.CompositionColorBrush";
}
unsafe impl Send for CompositionColorBrush {}
unsafe impl Sync for CompositionColorBrush {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionDrawingSurface(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionDrawingSurface,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionDrawingSurface,
    IAnimationObject,
    ICompositionSurface,
    CompositionObject
);
impl CompositionDrawingSurface {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionDrawingSurface {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionDrawingSurface>();
}
unsafe impl windows_core::Interface for CompositionDrawingSurface {
    type Vtable = <ICompositionDrawingSurface as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionDrawingSurface as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionDrawingSurface {
    const NAME: &'static str = "Microsoft.UI.Composition.CompositionDrawingSurface";
}
unsafe impl Send for CompositionDrawingSurface {}
unsafe impl Sync for CompositionDrawingSurface {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionGeometricClip(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionGeometricClip,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionGeometricClip,
    CompositionClip,
    CompositionObject
);
impl CompositionGeometricClip {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionGeometricClip {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionGeometricClip>();
}
unsafe impl windows_core::Interface for CompositionGeometricClip {
    type Vtable = <ICompositionGeometricClip as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionGeometricClip as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionGeometricClip {
    const NAME: &'static str = "Microsoft.UI.Composition.CompositionGeometricClip";
}
unsafe impl Send for CompositionGeometricClip {}
unsafe impl Sync for CompositionGeometricClip {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionGeometry(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionGeometry,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionGeometry, IAnimationObject, CompositionObject);
impl CompositionGeometry {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionGeometry {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionGeometry>();
}
unsafe impl windows_core::Interface for CompositionGeometry {
    type Vtable = <ICompositionGeometry as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionGeometry as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionGeometry {
    const NAME: &'static str = "Microsoft.UI.Composition.CompositionGeometry";
}
unsafe impl Send for CompositionGeometry {}
unsafe impl Sync for CompositionGeometry {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionGraphicsDevice(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionGraphicsDevice,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionGraphicsDevice, CompositionObject);
impl CompositionGraphicsDevice {
    pub fn CreateDrawingSurface(
        &self,
        sizepixels: Size,
        pixelformat: DirectXPixelFormat,
        alphamode: DirectXAlphaMode,
    ) -> windows_core::Result<CompositionDrawingSurface> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateDrawingSurface)(
                windows_core::Interface::as_raw(self),
                sizepixels,
                pixelformat,
                alphamode,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionGraphicsDevice {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionGraphicsDevice>();
}
unsafe impl windows_core::Interface for CompositionGraphicsDevice {
    type Vtable = <ICompositionGraphicsDevice as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionGraphicsDevice as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionGraphicsDevice {
    const NAME: &'static str = "Microsoft.UI.Composition.CompositionGraphicsDevice";
}
unsafe impl Send for CompositionGraphicsDevice {}
unsafe impl Sync for CompositionGraphicsDevice {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionInteractionSourceCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionInteractionSourceCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionInteractionSourceCollection, CompositionObject);
impl CompositionInteractionSourceCollection {
    pub fn Add<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionInteractionSource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Add)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionInteractionSourceCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionInteractionSourceCollection>(
        );
}
unsafe impl windows_core::Interface for CompositionInteractionSourceCollection {
    type Vtable = <ICompositionInteractionSourceCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <ICompositionInteractionSourceCollection as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionInteractionSourceCollection {
    const NAME: &'static str =
        "Microsoft.UI.Composition.Interactions.CompositionInteractionSourceCollection";
}
unsafe impl Send for CompositionInteractionSourceCollection {}
unsafe impl Sync for CompositionInteractionSourceCollection {}
impl IntoIterator for CompositionInteractionSourceCollection {
    type Item = ICompositionInteractionSource;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(&self)
    }
}
impl IntoIterator for &CompositionInteractionSourceCollection {
    type Item = ICompositionInteractionSource;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        windows_collections::BufferedIterator::new(self.First().unwrap())
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionObject(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionObject,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionObject, IAnimationObject);
impl CompositionObject {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).StartAnimation)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub fn StartAnimationWithIAnimationObject<P0, P2>(
        target: P0,
        propertyname: &windows_core::HSTRING,
        animation: P2,
    ) -> windows_core::Result<()>
    where
        P0: windows_core::Param<IAnimationObject>,
        P2: windows_core::Param<CompositionAnimation>,
    {
        Self::ICompositionObjectStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).StartAnimationWithIAnimationObject)(
                windows_core::Interface::as_raw(this),
                target.param().abi(),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        })
    }
    pub fn StartAnimationGroupWithIAnimationObject<P0, P1>(
        target: P0,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P0: windows_core::Param<IAnimationObject>,
        P1: windows_core::Param<ICompositionAnimationBase>,
    {
        Self::ICompositionObjectStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroupWithIAnimationObject)(
                windows_core::Interface::as_raw(this),
                target.param().abi(),
                animation.param().abi(),
            )
            .ok()
        })
    }
    fn ICompositionObjectStatics<
        R,
        F: FnOnce(&ICompositionObjectStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            CompositionObject,
            ICompositionObjectStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for CompositionObject {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionObject>();
}
unsafe impl windows_core::Interface for CompositionObject {
    type Vtable = <ICompositionObject as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionObject as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionObject {
    const NAME: &'static str = "Microsoft.UI.Composition.CompositionObject";
}
unsafe impl Send for CompositionObject {}
unsafe impl Sync for CompositionObject {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionRoundedRectangleGeometry(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionRoundedRectangleGeometry,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionRoundedRectangleGeometry,
    CompositionGeometry,
    CompositionObject
);
impl CompositionRoundedRectangleGeometry {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub fn SetCornerRadius(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetCornerRadius)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn SetOffset(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetOffset)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn SetSize(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetSize)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionRoundedRectangleGeometry {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionRoundedRectangleGeometry>();
}
unsafe impl windows_core::Interface for CompositionRoundedRectangleGeometry {
    type Vtable = <ICompositionRoundedRectangleGeometry as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <ICompositionRoundedRectangleGeometry as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionRoundedRectangleGeometry {
    const NAME: &'static str = "Microsoft.UI.Composition.CompositionRoundedRectangleGeometry";
}
unsafe impl Send for CompositionRoundedRectangleGeometry {}
unsafe impl Sync for CompositionRoundedRectangleGeometry {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionSurfaceBrush(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionSurfaceBrush,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionSurfaceBrush,
    CompositionBrush,
    CompositionObject
);
impl CompositionSurfaceBrush {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionSurfaceBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionSurfaceBrush>();
}
unsafe impl windows_core::Interface for CompositionSurfaceBrush {
    type Vtable = <ICompositionSurfaceBrush as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionSurfaceBrush as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionSurfaceBrush {
    const NAME: &'static str = "Microsoft.UI.Composition.CompositionSurfaceBrush";
}
unsafe impl Send for CompositionSurfaceBrush {}
unsafe impl Sync for CompositionSurfaceBrush {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Compositor(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    Compositor,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl Compositor {
    pub fn new() -> windows_core::Result<Self> {
        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
    }
    fn IActivationFactory<
        R,
        F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            Compositor,
            windows_core::imp::IGenericFactory,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    pub fn CreateColorBrushWithColor(
        &self,
        color: Color,
    ) -> windows_core::Result<CompositionColorBrush> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateColorBrushWithColor)(
                windows_core::Interface::as_raw(self),
                color,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn CreateContainerVisual(&self) -> windows_core::Result<ContainerVisual> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateContainerVisual)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn CreateExpressionAnimationWithExpression(
        &self,
        expression: &windows_core::HSTRING,
    ) -> windows_core::Result<ExpressionAnimation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateExpressionAnimationWithExpression)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(expression),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn CreateInsetClip(&self) -> windows_core::Result<InsetClip> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateInsetClip)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn CreateSpriteVisual(&self) -> windows_core::Result<SpriteVisual> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateSpriteVisual)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn CreateSurfaceBrushWithSurface<P0>(
        &self,
        surface: P0,
    ) -> windows_core::Result<CompositionSurfaceBrush>
    where
        P0: windows_core::Param<ICompositionSurface>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateSurfaceBrushWithSurface)(
                windows_core::Interface::as_raw(self),
                surface.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn CreateRoundedRectangleGeometry(
        &self,
    ) -> windows_core::Result<CompositionRoundedRectangleGeometry> {
        let this = &windows_core::Interface::cast::<ICompositor5>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateRoundedRectangleGeometry)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn CreateGeometricClipWithGeometry<P0>(
        &self,
        geometry: P0,
    ) -> windows_core::Result<CompositionGeometricClip>
    where
        P0: windows_core::Param<CompositionGeometry>,
    {
        let this = &windows_core::Interface::cast::<ICompositor6>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateGeometricClipWithGeometry)(
                windows_core::Interface::as_raw(this),
                geometry.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn MaxGlobalPlaybackRate() -> windows_core::Result<f32> {
        Self::ICompositorStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).MaxGlobalPlaybackRate)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    pub fn MinGlobalPlaybackRate() -> windows_core::Result<f32> {
        Self::ICompositorStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).MinGlobalPlaybackRate)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    fn ICompositorStatics<R, F: FnOnce(&ICompositorStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<Compositor, ICompositorStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for Compositor {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositor>();
}
unsafe impl windows_core::Interface for Compositor {
    type Vtable = <ICompositor as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositor as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for Compositor {
    const NAME: &'static str = "Microsoft.UI.Composition.Compositor";
}
unsafe impl Send for Compositor {}
unsafe impl Sync for Compositor {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerVisual(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ContainerVisual,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(ContainerVisual, Visual, CompositionObject);
impl ContainerVisual {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub fn Children(&self) -> windows_core::Result<VisualCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Children)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetClip<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<CompositionClip>,
    {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetClip)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn SetIsVisible(&self, value: bool) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIsVisible)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub fn SetOffset(&self, value: windows_numerics::Vector3) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOffset)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub fn SetSize(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetSize)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for ContainerVisual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IContainerVisual>();
}
unsafe impl windows_core::Interface for ContainerVisual {
    type Vtable = <IContainerVisual as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IContainerVisual as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for ContainerVisual {
    const NAME: &'static str = "Microsoft.UI.Composition.ContainerVisual";
}
unsafe impl Send for ContainerVisual {}
unsafe impl Sync for ContainerVisual {}
pub type D2D1_ANTIALIAS_MODE = i32;
pub const D2D1_ANTIALIAS_MODE_PER_PRIMITIVE: D2D1_ANTIALIAS_MODE = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D_RECT_F {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D_SIZE_F {
    pub width: f32,
    pub height: f32,
}
pub type DWRITE_FACTORY_TYPE = i32;
pub const DWRITE_FACTORY_TYPE_SHARED: DWRITE_FACTORY_TYPE = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DWRITE_FONT_METRICS {
    pub designUnitsPerEm: u16,
    pub ascent: u16,
    pub descent: u16,
    pub lineGap: i16,
    pub capHeight: u16,
    pub xHeight: u16,
    pub underlinePosition: i16,
    pub underlineThickness: u16,
    pub strikethroughPosition: i16,
    pub strikethroughThickness: u16,
}
pub type DWRITE_FONT_STRETCH = i32;
pub const DWRITE_FONT_STRETCH_NORMAL: DWRITE_FONT_STRETCH = 5;
pub type DWRITE_FONT_STYLE = i32;
pub const DWRITE_FONT_STYLE_NORMAL: DWRITE_FONT_STYLE = 0;
pub type DWRITE_FONT_WEIGHT = i32;
pub const DWRITE_FONT_WEIGHT_NORMAL: DWRITE_FONT_WEIGHT = 400;
pub const DWRITE_FONT_WEIGHT_SEMI_BOLD: DWRITE_FONT_WEIGHT = 600;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DWRITE_GLYPH_METRICS {
    pub leftSideBearing: i32,
    pub advanceWidth: u32,
    pub rightSideBearing: i32,
    pub topSideBearing: i32,
    pub advanceHeight: u32,
    pub bottomSideBearing: i32,
    pub verticalOriginY: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DWRITE_GLYPH_OFFSET {
    pub advanceOffset: f32,
    pub ascenderOffset: f32,
}
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DWRITE_GLYPH_RUN {
    pub fontFace: core::mem::ManuallyDrop<Option<IDWriteFontFace>>,
    pub fontEmSize: f32,
    pub glyphCount: u32,
    pub glyphIndices: *const u16,
    pub glyphAdvances: *const f32,
    pub glyphOffsets: *const DWRITE_GLYPH_OFFSET,
    pub isSideways: windows_core::BOOL,
    pub bidiLevel: u32,
}
pub type DWRITE_MEASURING_MODE = i32;
pub const DWRITE_MEASURING_MODE_NATURAL: DWRITE_MEASURING_MODE = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DWRITE_TRIMMING {
    pub granularity: DWRITE_TRIMMING_GRANULARITY,
    pub delimiter: u32,
    pub delimiterCount: u32,
}
pub type DWRITE_TRIMMING_GRANULARITY = i32;
pub const DWRITE_TRIMMING_GRANULARITY_CHARACTER: DWRITE_TRIMMING_GRANULARITY = 1;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DirectXAlphaMode(pub i32);
impl DirectXAlphaMode {
    pub const Unspecified: Self = Self(0);
    pub const Premultiplied: Self = Self(1);
    pub const Straight: Self = Self(2);
    pub const Ignore: Self = Self(3);
}
impl windows_core::imp::TypeKind for DirectXAlphaMode {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for DirectXAlphaMode {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Microsoft.Graphics.DirectX.DirectXAlphaMode;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.Graphics.DirectX.DirectXAlphaMode");
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DirectXPixelFormat(pub i32);
impl DirectXPixelFormat {
    pub const Unknown: Self = Self(0);
    pub const R32G32B32A32Typeless: Self = Self(1);
    pub const R32G32B32A32Float: Self = Self(2);
    pub const R32G32B32A32UInt: Self = Self(3);
    pub const R32G32B32A32Int: Self = Self(4);
    pub const R32G32B32Typeless: Self = Self(5);
    pub const R32G32B32Float: Self = Self(6);
    pub const R32G32B32UInt: Self = Self(7);
    pub const R32G32B32Int: Self = Self(8);
    pub const R16G16B16A16Typeless: Self = Self(9);
    pub const R16G16B16A16Float: Self = Self(10);
    pub const R16G16B16A16UIntNormalized: Self = Self(11);
    pub const R16G16B16A16UInt: Self = Self(12);
    pub const R16G16B16A16IntNormalized: Self = Self(13);
    pub const R16G16B16A16Int: Self = Self(14);
    pub const R32G32Typeless: Self = Self(15);
    pub const R32G32Float: Self = Self(16);
    pub const R32G32UInt: Self = Self(17);
    pub const R32G32Int: Self = Self(18);
    pub const R32G8X24Typeless: Self = Self(19);
    pub const D32FloatS8X24UInt: Self = Self(20);
    pub const R32FloatX8X24Typeless: Self = Self(21);
    pub const X32TypelessG8X24UInt: Self = Self(22);
    pub const R10G10B10A2Typeless: Self = Self(23);
    pub const R10G10B10A2UIntNormalized: Self = Self(24);
    pub const R10G10B10A2UInt: Self = Self(25);
    pub const R11G11B10Float: Self = Self(26);
    pub const R8G8B8A8Typeless: Self = Self(27);
    pub const R8G8B8A8UIntNormalized: Self = Self(28);
    pub const R8G8B8A8UIntNormalizedSrgb: Self = Self(29);
    pub const R8G8B8A8UInt: Self = Self(30);
    pub const R8G8B8A8IntNormalized: Self = Self(31);
    pub const R8G8B8A8Int: Self = Self(32);
    pub const R16G16Typeless: Self = Self(33);
    pub const R16G16Float: Self = Self(34);
    pub const R16G16UIntNormalized: Self = Self(35);
    pub const R16G16UInt: Self = Self(36);
    pub const R16G16IntNormalized: Self = Self(37);
    pub const R16G16Int: Self = Self(38);
    pub const R32Typeless: Self = Self(39);
    pub const D32Float: Self = Self(40);
    pub const R32Float: Self = Self(41);
    pub const R32UInt: Self = Self(42);
    pub const R32Int: Self = Self(43);
    pub const R24G8Typeless: Self = Self(44);
    pub const D24UIntNormalizedS8UInt: Self = Self(45);
    pub const R24UIntNormalizedX8Typeless: Self = Self(46);
    pub const X24TypelessG8UInt: Self = Self(47);
    pub const R8G8Typeless: Self = Self(48);
    pub const R8G8UIntNormalized: Self = Self(49);
    pub const R8G8UInt: Self = Self(50);
    pub const R8G8IntNormalized: Self = Self(51);
    pub const R8G8Int: Self = Self(52);
    pub const R16Typeless: Self = Self(53);
    pub const R16Float: Self = Self(54);
    pub const D16UIntNormalized: Self = Self(55);
    pub const R16UIntNormalized: Self = Self(56);
    pub const R16UInt: Self = Self(57);
    pub const R16IntNormalized: Self = Self(58);
    pub const R16Int: Self = Self(59);
    pub const R8Typeless: Self = Self(60);
    pub const R8UIntNormalized: Self = Self(61);
    pub const R8UInt: Self = Self(62);
    pub const R8IntNormalized: Self = Self(63);
    pub const R8Int: Self = Self(64);
    pub const A8UIntNormalized: Self = Self(65);
    pub const R1UIntNormalized: Self = Self(66);
    pub const R9G9B9E5SharedExponent: Self = Self(67);
    pub const R8G8B8G8UIntNormalized: Self = Self(68);
    pub const G8R8G8B8UIntNormalized: Self = Self(69);
    pub const BC1Typeless: Self = Self(70);
    pub const BC1UIntNormalized: Self = Self(71);
    pub const BC1UIntNormalizedSrgb: Self = Self(72);
    pub const BC2Typeless: Self = Self(73);
    pub const BC2UIntNormalized: Self = Self(74);
    pub const BC2UIntNormalizedSrgb: Self = Self(75);
    pub const BC3Typeless: Self = Self(76);
    pub const BC3UIntNormalized: Self = Self(77);
    pub const BC3UIntNormalizedSrgb: Self = Self(78);
    pub const BC4Typeless: Self = Self(79);
    pub const BC4UIntNormalized: Self = Self(80);
    pub const BC4IntNormalized: Self = Self(81);
    pub const BC5Typeless: Self = Self(82);
    pub const BC5UIntNormalized: Self = Self(83);
    pub const BC5IntNormalized: Self = Self(84);
    pub const B5G6R5UIntNormalized: Self = Self(85);
    pub const B5G5R5A1UIntNormalized: Self = Self(86);
    pub const B8G8R8A8UIntNormalized: Self = Self(87);
    pub const B8G8R8X8UIntNormalized: Self = Self(88);
    pub const R10G10B10XRBiasA2UIntNormalized: Self = Self(89);
    pub const B8G8R8A8Typeless: Self = Self(90);
    pub const B8G8R8A8UIntNormalizedSrgb: Self = Self(91);
    pub const B8G8R8X8Typeless: Self = Self(92);
    pub const B8G8R8X8UIntNormalizedSrgb: Self = Self(93);
    pub const BC6HTypeless: Self = Self(94);
    pub const BC6H16UnsignedFloat: Self = Self(95);
    pub const BC6H16Float: Self = Self(96);
    pub const BC7Typeless: Self = Self(97);
    pub const BC7UIntNormalized: Self = Self(98);
    pub const BC7UIntNormalizedSrgb: Self = Self(99);
    pub const Ayuv: Self = Self(100);
    pub const Y410: Self = Self(101);
    pub const Y416: Self = Self(102);
    pub const NV12: Self = Self(103);
    pub const P010: Self = Self(104);
    pub const P016: Self = Self(105);
    pub const Opaque420: Self = Self(106);
    pub const Yuy2: Self = Self(107);
    pub const Y210: Self = Self(108);
    pub const Y216: Self = Self(109);
    pub const NV11: Self = Self(110);
    pub const AI44: Self = Self(111);
    pub const IA44: Self = Self(112);
    pub const P8: Self = Self(113);
    pub const A8P8: Self = Self(114);
    pub const B4G4R4A4UIntNormalized: Self = Self(115);
    pub const P208: Self = Self(130);
    pub const V208: Self = Self(131);
    pub const V408: Self = Self(132);
    pub const SamplerFeedbackMinMipOpaque: Self = Self(189);
    pub const SamplerFeedbackMipRegionUsedOpaque: Self = Self(190);
    pub const A4B4G4R4: Self = Self(191);
}
impl windows_core::imp::TypeKind for DirectXPixelFormat {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for DirectXPixelFormat {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Microsoft.Graphics.DirectX.DirectXPixelFormat;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.Graphics.DirectX.DirectXPixelFormat",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpressionAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ExpressionAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    ExpressionAnimation,
    CompositionAnimation,
    CompositionObject
);
impl ExpressionAnimation {
    pub fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for ExpressionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IExpressionAnimation>();
}
unsafe impl windows_core::Interface for ExpressionAnimation {
    type Vtable = <IExpressionAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IExpressionAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for ExpressionAnimation {
    const NAME: &'static str = "Microsoft.UI.Composition.ExpressionAnimation";
}
unsafe impl Send for ExpressionAnimation {}
unsafe impl Sync for ExpressionAnimation {}
windows_core::imp::define_interface!(
    IAnimationObject,
    IAnimationObject_Vtbl,
    0x8f56119d_b96d_58d0_9916_d1c5e390f890
);
impl windows_core::RuntimeType for IAnimationObject {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.IAnimationObject");
}
windows_core::imp::interface_hierarchy!(
    IAnimationObject,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for IAnimationObject {
    const NAME: &'static str = "Microsoft.UI.Composition.IAnimationObject";
}
#[repr(C)]
pub struct IAnimationObject_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionAnimation,
    ICompositionAnimation_Vtbl,
    0xa829ccc8_6fde_5b90_ad37_efd307e1b631
);
impl windows_core::RuntimeType for ICompositionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.ICompositionAnimation",
    );
}
#[repr(C)]
pub struct ICompositionAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    ClearAllParameters: usize,
    ClearParameter: usize,
    SetColorParameter: usize,
    SetMatrix3x2Parameter: usize,
    SetMatrix4x4Parameter: usize,
    SetQuaternionParameter: usize,
    pub SetReferenceParameter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetScalarParameter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        f32,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionAnimationBase,
    ICompositionAnimationBase_Vtbl,
    0xa77c0e5a_f059_4e85_bcef_c068694cec78
);
impl windows_core::RuntimeType for ICompositionAnimationBase {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.ICompositionAnimationBase",
    );
}
windows_core::imp::interface_hierarchy!(
    ICompositionAnimationBase,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for ICompositionAnimationBase {
    const NAME: &'static str = "Microsoft.UI.Composition.ICompositionAnimationBase";
}
pub trait ICompositionAnimationBase_Impl: windows_core::IUnknownImpl {}
impl ICompositionAnimationBase_Vtbl {
    pub const fn new<Identity: ICompositionAnimationBase_Impl, const OFFSET: isize>() -> Self {
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<
                Identity,
                ICompositionAnimationBase,
                OFFSET,
            >(),
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ICompositionAnimationBase as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct ICompositionAnimationBase_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionBrush,
    ICompositionBrush_Vtbl,
    0x483924e7_99a5_5377_968b_dec6d40bbccd
);
impl windows_core::RuntimeType for ICompositionBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.ICompositionBrush");
}
#[repr(C)]
pub struct ICompositionBrush_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionClip,
    ICompositionClip_Vtbl,
    0xb66b55cb_b5a5_5bee_8972_ae78233cb34c
);
impl windows_core::RuntimeType for ICompositionClip {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.ICompositionClip");
}
#[repr(C)]
pub struct ICompositionClip_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionColorBrush,
    ICompositionColorBrush_Vtbl,
    0x3f8ffb69_3e71_55a7_8e79_f27a214c56ae
);
impl windows_core::RuntimeType for ICompositionColorBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.ICompositionColorBrush",
    );
}
#[repr(C)]
pub struct ICompositionColorBrush_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    Color: usize,
    pub SetColor: unsafe extern "system" fn(*mut core::ffi::c_void, Color) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionDrawingSurface,
    ICompositionDrawingSurface_Vtbl,
    0x216cab97_a2ee_5a29_ad6b_0bc2df4a1504
);
impl windows_core::RuntimeType for ICompositionDrawingSurface {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.ICompositionDrawingSurface",
    );
}
#[repr(C)]
pub struct ICompositionDrawingSurface_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionGeometricClip,
    ICompositionGeometricClip_Vtbl,
    0xe3c0a8c5_5297_5d69_8b91_f5330bb1cb27
);
impl windows_core::RuntimeType for ICompositionGeometricClip {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.ICompositionGeometricClip",
    );
}
#[repr(C)]
pub struct ICompositionGeometricClip_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionGeometry,
    ICompositionGeometry_Vtbl,
    0x4e40bdb2_450b_5a81_9e9b_149417980cc4
);
impl windows_core::RuntimeType for ICompositionGeometry {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.ICompositionGeometry",
    );
}
#[repr(C)]
pub struct ICompositionGeometry_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionGraphicsDevice,
    ICompositionGraphicsDevice_Vtbl,
    0x3d47e3f5_f76c_5f1f_88c0_54a5f2a090d6
);
impl windows_core::RuntimeType for ICompositionGraphicsDevice {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.ICompositionGraphicsDevice",
    );
}
#[repr(C)]
pub struct ICompositionGraphicsDevice_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateDrawingSurface: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        Size,
        DirectXPixelFormat,
        DirectXAlphaMode,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionInteractionSource,
    ICompositionInteractionSource_Vtbl,
    0x711c72c0_c406_4a12_859b_b44f651af046
);
impl windows_core::RuntimeType for ICompositionInteractionSource {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.ICompositionInteractionSource",
    );
}
windows_core::imp::interface_hierarchy!(
    ICompositionInteractionSource,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for ICompositionInteractionSource {
    const NAME: &'static str =
        "Microsoft.UI.Composition.Interactions.ICompositionInteractionSource";
}
pub trait ICompositionInteractionSource_Impl: windows_core::IUnknownImpl {}
impl ICompositionInteractionSource_Vtbl {
    pub const fn new<Identity: ICompositionInteractionSource_Impl, const OFFSET: isize>() -> Self {
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<
                Identity,
                ICompositionInteractionSource,
                OFFSET,
            >(),
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ICompositionInteractionSource as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct ICompositionInteractionSource_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionInteractionSourceCollection,
    ICompositionInteractionSourceCollection_Vtbl,
    0x9aa1b86b_b002_5e2e_bb2b_0e2c547445e1
);
impl windows_core::RuntimeType for ICompositionInteractionSourceCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.ICompositionInteractionSourceCollection",
    );
}
#[repr(C)]
pub struct ICompositionInteractionSourceCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    Count: usize,
    pub Add: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionObject,
    ICompositionObject_Vtbl,
    0x0e583d49_fb5e_5481_a426_d3c41e059a5a
);
impl windows_core::RuntimeType for ICompositionObject {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.ICompositionObject");
}
#[repr(C)]
pub struct ICompositionObject_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    Compositor: usize,
    Properties: usize,
    pub StartAnimation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionObjectStatics,
    ICompositionObjectStatics_Vtbl,
    0xa091356c_38ae_514c_9b58_ad5c8c7936ad
);
impl windows_core::RuntimeType for ICompositionObjectStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.ICompositionObjectStatics",
    );
}
#[repr(C)]
pub struct ICompositionObjectStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub StartAnimationWithIAnimationObject: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub StartAnimationGroupWithIAnimationObject: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    )
        -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionRoundedRectangleGeometry,
    ICompositionRoundedRectangleGeometry_Vtbl,
    0x02eafc87_8d1f_5445_a416_d81baee8a750
);
impl windows_core::RuntimeType for ICompositionRoundedRectangleGeometry {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.ICompositionRoundedRectangleGeometry",
    );
}
#[repr(C)]
pub struct ICompositionRoundedRectangleGeometry_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    CornerRadius: usize,
    pub SetCornerRadius: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    Offset: usize,
    pub SetOffset: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    Size: usize,
    pub SetSize: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionSurface,
    ICompositionSurface_Vtbl,
    0x9ec612c3_a5d2_4f97_9df3_6b49ce736215
);
impl windows_core::RuntimeType for ICompositionSurface {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.ICompositionSurface");
}
windows_core::imp::interface_hierarchy!(
    ICompositionSurface,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for ICompositionSurface {
    const NAME: &'static str = "Microsoft.UI.Composition.ICompositionSurface";
}
pub trait ICompositionSurface_Impl: windows_core::IUnknownImpl {}
impl ICompositionSurface_Vtbl {
    pub const fn new<Identity: ICompositionSurface_Impl, const OFFSET: isize>() -> Self {
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, ICompositionSurface, OFFSET>(),
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ICompositionSurface as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct ICompositionSurface_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionSurfaceBrush,
    ICompositionSurfaceBrush_Vtbl,
    0x616bb5a5_0a33_512d_b4b1_3d3734f04aca
);
impl windows_core::RuntimeType for ICompositionSurfaceBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.ICompositionSurfaceBrush",
    );
}
#[repr(C)]
pub struct ICompositionSurfaceBrush_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositor,
    ICompositor_Vtbl,
    0x95213c13_c4cb_57de_b267_d21ab901ae38
);
impl windows_core::RuntimeType for ICompositor {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.ICompositor");
}
#[repr(C)]
pub struct ICompositor_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    CreateColorKeyFrameAnimation: usize,
    CreateColorBrush: usize,
    pub CreateColorBrushWithColor: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        Color,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateContainerVisual: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    CreateCubicBezierEasingFunction: usize,
    CreateEffectFactory: usize,
    CreateEffectFactoryWithProperties: usize,
    CreateExpressionAnimation: usize,
    pub CreateExpressionAnimationWithExpression: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    )
        -> windows_core::HRESULT,
    pub CreateInsetClip: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    CreateInsetClipWithInsets: usize,
    CreateLinearEasingFunction: usize,
    CreatePropertySet: usize,
    CreateQuaternionKeyFrameAnimation: usize,
    CreateScalarKeyFrameAnimation: usize,
    CreateScopedBatch: usize,
    pub CreateSpriteVisual: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    CreateSurfaceBrush: usize,
    pub CreateSurfaceBrushWithSurface: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositor5,
    ICompositor5_Vtbl,
    0xbb52d644_a030_5c19_b883_577ded739ae7
);
impl windows_core::RuntimeType for ICompositor5 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.ICompositor5");
}
#[repr(C)]
pub struct ICompositor5_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    Comment: usize,
    SetComment: usize,
    GlobalPlaybackRate: usize,
    SetGlobalPlaybackRate: usize,
    CreateBounceScalarAnimation: usize,
    CreateBounceVector2Animation: usize,
    CreateBounceVector3Animation: usize,
    CreateContainerShape: usize,
    CreateEllipseGeometry: usize,
    CreateLineGeometry: usize,
    CreatePathGeometry: usize,
    CreatePathGeometryWithPath: usize,
    CreatePathKeyFrameAnimation: usize,
    CreateRectangleGeometry: usize,
    pub CreateRoundedRectangleGeometry: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositor6,
    ICompositor6_Vtbl,
    0x5fd20200_c188_5737_b567_5e61741af19c
);
impl windows_core::RuntimeType for ICompositor6 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.ICompositor6");
}
#[repr(C)]
pub struct ICompositor6_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    CreateGeometricClip: usize,
    pub CreateGeometricClipWithGeometry: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositorStatics,
    ICompositorStatics_Vtbl,
    0x6baa947a_f103_55c6_91ad_3d275bea65c4
);
impl windows_core::RuntimeType for ICompositorStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.ICompositorStatics");
}
#[repr(C)]
pub struct ICompositorStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub MaxGlobalPlaybackRate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub MinGlobalPlaybackRate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IContainerVisual,
    IContainerVisual_Vtbl,
    0xc70dbce1_2c2f_5d8e_91a4_aae1121e6186
);
impl windows_core::RuntimeType for IContainerVisual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.IContainerVisual");
}
#[repr(C)]
pub struct IContainerVisual_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Children: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ID2D1Brush,
    ID2D1Brush_Vtbl,
    0x2cd906a8_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1Brush {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1Brush, windows_core::IUnknown, ID2D1Resource);
impl ID2D1Brush {
    pub unsafe fn SetOpacity(&self, opacity: f32) {
        unsafe {
            (windows_core::Interface::vtable(self).SetOpacity)(
                windows_core::Interface::as_raw(self),
                opacity,
            );
        }
    }
    pub unsafe fn SetTransform(&self, transform: *const windows_numerics::Matrix3x2) {
        unsafe {
            (windows_core::Interface::vtable(self).SetTransform)(
                windows_core::Interface::as_raw(self),
                transform,
            );
        }
    }
    pub unsafe fn GetOpacity(&self) -> f32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetOpacity)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub unsafe fn GetTransform(&self, transform: *mut windows_numerics::Matrix3x2) {
        unsafe {
            (windows_core::Interface::vtable(self).GetTransform)(
                windows_core::Interface::as_raw(self),
                transform as _,
            );
        }
    }
}
#[repr(C)]
pub struct ID2D1Brush_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    pub SetOpacity: unsafe extern "system" fn(*mut core::ffi::c_void, f32),
    pub SetTransform:
        unsafe extern "system" fn(*mut core::ffi::c_void, *const windows_numerics::Matrix3x2),
    pub GetOpacity: unsafe extern "system" fn(*mut core::ffi::c_void) -> f32,
    pub GetTransform:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_numerics::Matrix3x2),
}
impl windows_core::RuntimeName for ID2D1Brush {}
windows_core::imp::define_interface!(
    ID2D1DeviceContext,
    ID2D1DeviceContext_Vtbl,
    0xe8f7fe7a_191c_466d_ad95_975678bda998
);
impl core::ops::Deref for ID2D1DeviceContext {
    type Target = ID2D1RenderTarget;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1DeviceContext,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1RenderTarget
);
#[repr(C)]
pub struct ID2D1DeviceContext_Vtbl {
    pub base__: ID2D1RenderTarget_Vtbl,
    CreateBitmap: usize,
    CreateBitmapFromWicBitmap: usize,
    CreateColorContext: usize,
    CreateColorContextFromFilename: usize,
    CreateColorContextFromWicColorContext: usize,
    CreateBitmapFromDxgiSurface: usize,
    CreateEffect: usize,
    CreateGradientStopCollection: usize,
    CreateImageBrush: usize,
    CreateBitmapBrush: usize,
    CreateCommandList: usize,
    IsDxgiFormatSupported: usize,
    IsBufferPrecisionSupported: usize,
    GetImageLocalBounds: usize,
    GetImageWorldBounds: usize,
    GetGlyphRunWorldBounds: usize,
    GetDevice: usize,
    SetTarget: usize,
    GetTarget: usize,
    SetRenderingControls: usize,
    GetRenderingControls: usize,
    SetPrimitiveBlend: usize,
    GetPrimitiveBlend: usize,
    SetUnitMode: usize,
    GetUnitMode: usize,
    DrawGlyphRun: usize,
    DrawImage: usize,
    DrawGdiMetafile: usize,
    DrawBitmap: usize,
    PushLayer: usize,
    InvalidateEffectInputRectangle: usize,
    GetEffectInvalidRectangleCount: usize,
    GetEffectInvalidRectangles: usize,
    GetEffectRequiredInputRectangles: usize,
    FillOpacityMask: usize,
}
impl windows_core::RuntimeName for ID2D1DeviceContext {}
windows_core::imp::define_interface!(
    ID2D1DeviceContext1,
    ID2D1DeviceContext1_Vtbl,
    0xd37f57e4_6908_459f_a199_e72f24f79987
);
impl core::ops::Deref for ID2D1DeviceContext1 {
    type Target = ID2D1DeviceContext;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1DeviceContext1,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1RenderTarget,
    ID2D1DeviceContext
);
#[repr(C)]
pub struct ID2D1DeviceContext1_Vtbl {
    pub base__: ID2D1DeviceContext_Vtbl,
    CreateFilledGeometryRealization: usize,
    CreateStrokedGeometryRealization: usize,
    DrawGeometryRealization: usize,
}
impl windows_core::RuntimeName for ID2D1DeviceContext1 {}
windows_core::imp::define_interface!(
    ID2D1DeviceContext2,
    ID2D1DeviceContext2_Vtbl,
    0x394ea6a3_0c34_4321_950b_6ca20f0be6c7
);
impl core::ops::Deref for ID2D1DeviceContext2 {
    type Target = ID2D1DeviceContext1;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1DeviceContext2,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1RenderTarget,
    ID2D1DeviceContext,
    ID2D1DeviceContext1
);
#[repr(C)]
pub struct ID2D1DeviceContext2_Vtbl {
    pub base__: ID2D1DeviceContext1_Vtbl,
    CreateInk: usize,
    CreateInkStyle: usize,
    CreateGradientMesh: usize,
    CreateImageSourceFromWic: usize,
    CreateLookupTable3D: usize,
    CreateImageSourceFromDxgi: usize,
    GetGradientMeshWorldBounds: usize,
    DrawInk: usize,
    DrawGradientMesh: usize,
    DrawGdiMetafile: usize,
    CreateTransformedImageSource: usize,
}
impl windows_core::RuntimeName for ID2D1DeviceContext2 {}
windows_core::imp::define_interface!(
    ID2D1DeviceContext3,
    ID2D1DeviceContext3_Vtbl,
    0x235a7496_8351_414c_bcd4_6672ab2d8e00
);
impl core::ops::Deref for ID2D1DeviceContext3 {
    type Target = ID2D1DeviceContext2;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1DeviceContext3,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1RenderTarget,
    ID2D1DeviceContext,
    ID2D1DeviceContext1,
    ID2D1DeviceContext2
);
#[repr(C)]
pub struct ID2D1DeviceContext3_Vtbl {
    pub base__: ID2D1DeviceContext2_Vtbl,
    CreateSpriteBatch: usize,
    DrawSpriteBatch: usize,
}
impl windows_core::RuntimeName for ID2D1DeviceContext3 {}
windows_core::imp::define_interface!(
    ID2D1DeviceContext4,
    ID2D1DeviceContext4_Vtbl,
    0x8c427831_3d90_4476_b647_c4fae349e4db
);
impl core::ops::Deref for ID2D1DeviceContext4 {
    type Target = ID2D1DeviceContext3;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1DeviceContext4,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1RenderTarget,
    ID2D1DeviceContext,
    ID2D1DeviceContext1,
    ID2D1DeviceContext2,
    ID2D1DeviceContext3
);
#[repr(C)]
pub struct ID2D1DeviceContext4_Vtbl {
    pub base__: ID2D1DeviceContext3_Vtbl,
    CreateSvgGlyphStyle: usize,
    DrawText: usize,
    DrawTextLayout: usize,
    DrawColorBitmapGlyphRun: usize,
    DrawSvgGlyphRun: usize,
    GetColorBitmapGlyphImage: usize,
    GetSvgGlyphImage: usize,
}
impl windows_core::RuntimeName for ID2D1DeviceContext4 {}
windows_core::imp::define_interface!(
    ID2D1DeviceContext5,
    ID2D1DeviceContext5_Vtbl,
    0x7836d248_68cc_4df6_b9e8_de991bf62eb7
);
impl core::ops::Deref for ID2D1DeviceContext5 {
    type Target = ID2D1DeviceContext4;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1DeviceContext5,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1RenderTarget,
    ID2D1DeviceContext,
    ID2D1DeviceContext1,
    ID2D1DeviceContext2,
    ID2D1DeviceContext3,
    ID2D1DeviceContext4
);
impl ID2D1DeviceContext5 {
    pub unsafe fn CreateSvgDocument<P0>(
        &self,
        inputxmlstream: P0,
        viewportsize: D2D_SIZE_F,
    ) -> windows_core::Result<ID2D1SvgDocument>
    where
        P0: windows_core::Param<IStream>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateSvgDocument)(
                windows_core::Interface::as_raw(self),
                inputxmlstream.param().abi(),
                viewportsize,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn DrawSvgDocument<P0>(&self, svgdocument: P0)
    where
        P0: windows_core::Param<ID2D1SvgDocument>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawSvgDocument)(
                windows_core::Interface::as_raw(self),
                svgdocument.param().abi(),
            );
        }
    }
}
#[repr(C)]
pub struct ID2D1DeviceContext5_Vtbl {
    pub base__: ID2D1DeviceContext4_Vtbl,
    pub CreateSvgDocument: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        D2D_SIZE_F,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub DrawSvgDocument: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    CreateColorContextFromDxgiColorSpace: usize,
    CreateColorContextFromSimpleColorProfile: usize,
}
impl windows_core::RuntimeName for ID2D1DeviceContext5 {}
windows_core::imp::define_interface!(
    ID2D1RenderTarget,
    ID2D1RenderTarget_Vtbl,
    0x2cd90694_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1RenderTarget {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1RenderTarget, windows_core::IUnknown, ID2D1Resource);
impl ID2D1RenderTarget {
    pub unsafe fn DrawGlyphRun<P2>(
        &self,
        baselineorigin: windows_numerics::Vector2,
        glyphrun: *const DWRITE_GLYPH_RUN,
        foregroundbrush: P2,
        measuringmode: DWRITE_MEASURING_MODE,
    ) where
        P2: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawGlyphRun)(
                windows_core::Interface::as_raw(self),
                baselineorigin,
                glyphrun,
                foregroundbrush.param().abi(),
                measuringmode,
            );
        }
    }
    pub unsafe fn PushAxisAlignedClip(
        &self,
        cliprect: *const D2D_RECT_F,
        antialiasmode: D2D1_ANTIALIAS_MODE,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).PushAxisAlignedClip)(
                windows_core::Interface::as_raw(self),
                cliprect,
                antialiasmode,
            );
        }
    }
    pub unsafe fn PopAxisAlignedClip(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).PopAxisAlignedClip)(
                windows_core::Interface::as_raw(self),
            );
        }
    }
}
#[repr(C)]
pub struct ID2D1RenderTarget_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    CreateBitmap: usize,
    CreateBitmapFromWicBitmap: usize,
    CreateSharedBitmap: usize,
    CreateBitmapBrush: usize,
    CreateSolidColorBrush: usize,
    CreateGradientStopCollection: usize,
    CreateLinearGradientBrush: usize,
    CreateRadialGradientBrush: usize,
    CreateCompatibleRenderTarget: usize,
    CreateLayer: usize,
    CreateMesh: usize,
    DrawLine: usize,
    DrawRectangle: usize,
    FillRectangle: usize,
    DrawRoundedRectangle: usize,
    FillRoundedRectangle: usize,
    DrawEllipse: usize,
    FillEllipse: usize,
    DrawGeometry: usize,
    FillGeometry: usize,
    FillMesh: usize,
    FillOpacityMask: usize,
    DrawBitmap: usize,
    DrawText: usize,
    DrawTextLayout: usize,
    pub DrawGlyphRun: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
        *const DWRITE_GLYPH_RUN,
        *mut core::ffi::c_void,
        DWRITE_MEASURING_MODE,
    ),
    SetTransform: usize,
    GetTransform: usize,
    SetAntialiasMode: usize,
    GetAntialiasMode: usize,
    SetTextAntialiasMode: usize,
    GetTextAntialiasMode: usize,
    SetTextRenderingParams: usize,
    GetTextRenderingParams: usize,
    SetTags: usize,
    GetTags: usize,
    PushLayer: usize,
    PopLayer: usize,
    Flush: usize,
    SaveDrawingState: usize,
    RestoreDrawingState: usize,
    pub PushAxisAlignedClip:
        unsafe extern "system" fn(*mut core::ffi::c_void, *const D2D_RECT_F, D2D1_ANTIALIAS_MODE),
    pub PopAxisAlignedClip: unsafe extern "system" fn(*mut core::ffi::c_void),
    Clear: usize,
    BeginDraw: usize,
    EndDraw: usize,
    GetPixelFormat: usize,
    SetDpi: usize,
    GetDpi: usize,
    GetSize: usize,
    GetPixelSize: usize,
    GetMaximumBitmapSize: usize,
    IsSupported: usize,
}
impl windows_core::RuntimeName for ID2D1RenderTarget {}
windows_core::imp::define_interface!(
    ID2D1Resource,
    ID2D1Resource_Vtbl,
    0x2cd90691_12e2_11dc_9fed_001143a055f9
);
windows_core::imp::interface_hierarchy!(ID2D1Resource, windows_core::IUnknown);
#[repr(C)]
pub struct ID2D1Resource_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetFactory: usize,
}
impl windows_core::RuntimeName for ID2D1Resource {}
windows_core::imp::define_interface!(
    ID2D1SvgDocument,
    ID2D1SvgDocument_Vtbl,
    0x86b88e4d_afa4_4d7b_88e4_68a51c4a0aec
);
impl core::ops::Deref for ID2D1SvgDocument {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1SvgDocument, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1SvgDocument_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    SetViewportSize: usize,
    GetViewportSize: usize,
    SetRoot: usize,
    GetRoot: usize,
    FindElementById: usize,
    Serialize: usize,
    Deserialize: usize,
    CreatePaint: usize,
    CreateStrokeDashArray: usize,
    CreatePointCollection: usize,
    CreatePathData: usize,
}
impl windows_core::RuntimeName for ID2D1SvgDocument {}
windows_core::imp::define_interface!(
    IDWriteFactory,
    IDWriteFactory_Vtbl,
    0xb859ee5a_d838_4b5b_a2e8_1adc7d93db48
);
windows_core::imp::interface_hierarchy!(IDWriteFactory, windows_core::IUnknown);
impl IDWriteFactory {
    pub unsafe fn GetSystemFontCollection(
        &self,
        fontcollection: *mut Option<IDWriteFontCollection>,
        checkforupdates: bool,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetSystemFontCollection)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute(fontcollection),
                checkforupdates.into(),
            )
        }
    }
    pub unsafe fn CreateEllipsisTrimmingSign<P0>(
        &self,
        textformat: P0,
    ) -> windows_core::Result<IDWriteInlineObject>
    where
        P0: windows_core::Param<IDWriteTextFormat>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateEllipsisTrimmingSign)(
                windows_core::Interface::as_raw(self),
                textformat.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDWriteFactory_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetSystemFontCollection: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
        windows_core::BOOL,
    ) -> windows_core::HRESULT,
    CreateCustomFontCollection: usize,
    RegisterFontCollectionLoader: usize,
    UnregisterFontCollectionLoader: usize,
    CreateFontFileReference: usize,
    CreateCustomFontFileReference: usize,
    CreateFontFace: usize,
    CreateRenderingParams: usize,
    CreateMonitorRenderingParams: usize,
    CreateCustomRenderingParams: usize,
    RegisterFontFileLoader: usize,
    UnregisterFontFileLoader: usize,
    CreateTextFormat: usize,
    CreateTypography: usize,
    GetGdiInterop: usize,
    CreateTextLayout: usize,
    CreateGdiCompatibleTextLayout: usize,
    pub CreateEllipsisTrimmingSign: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    CreateTextAnalyzer: usize,
    CreateNumberSubstitution: usize,
    CreateGlyphRunAnalysis: usize,
}
impl windows_core::RuntimeName for IDWriteFactory {}
windows_core::imp::define_interface!(
    IDWriteFont,
    IDWriteFont_Vtbl,
    0xacd16696_8c14_4f5d_877e_fe3fc1d32737
);
windows_core::imp::interface_hierarchy!(IDWriteFont, windows_core::IUnknown);
impl IDWriteFont {
    pub unsafe fn CreateFontFace(&self) -> windows_core::Result<IDWriteFontFace> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateFontFace)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDWriteFont_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetFontFamily: usize,
    GetWeight: usize,
    GetStretch: usize,
    GetStyle: usize,
    IsSymbolFont: usize,
    GetFaceNames: usize,
    GetInformationalStrings: usize,
    GetSimulations: usize,
    GetMetrics: usize,
    HasCharacter: usize,
    pub CreateFontFace: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for IDWriteFont {}
windows_core::imp::define_interface!(
    IDWriteFontCollection,
    IDWriteFontCollection_Vtbl,
    0xa84cee02_3eea_4eee_a827_87c1a02a0fcc
);
windows_core::imp::interface_hierarchy!(IDWriteFontCollection, windows_core::IUnknown);
impl IDWriteFontCollection {
    pub unsafe fn GetFontFamily(&self, index: u32) -> windows_core::Result<IDWriteFontFamily> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFontFamily)(
                windows_core::Interface::as_raw(self),
                index,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn FindFamilyName<P0>(
        &self,
        familyname: P0,
        index: *mut u32,
        exists: *mut windows_core::BOOL,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FindFamilyName)(
                windows_core::Interface::as_raw(self),
                familyname.param().abi(),
                index as _,
                exists as _,
            )
        }
    }
}
#[repr(C)]
pub struct IDWriteFontCollection_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetFontFamilyCount: usize,
    pub GetFontFamily: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub FindFamilyName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        *mut u32,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    GetFontFromFontFace: usize,
}
impl windows_core::RuntimeName for IDWriteFontCollection {}
windows_core::imp::define_interface!(
    IDWriteFontFace,
    IDWriteFontFace_Vtbl,
    0x5f49804d_7024_4d43_bfa9_d25984f53849
);
windows_core::imp::interface_hierarchy!(IDWriteFontFace, windows_core::IUnknown);
impl IDWriteFontFace {
    pub unsafe fn GetMetrics(&self, fontfacemetrics: *mut DWRITE_FONT_METRICS) {
        unsafe {
            (windows_core::Interface::vtable(self).GetMetrics)(
                windows_core::Interface::as_raw(self),
                fontfacemetrics as _,
            );
        }
    }
    pub unsafe fn GetDesignGlyphMetrics(
        &self,
        glyphindices: *const u16,
        glyphcount: u32,
        glyphmetrics: *mut DWRITE_GLYPH_METRICS,
        issideways: bool,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetDesignGlyphMetrics)(
                windows_core::Interface::as_raw(self),
                glyphindices,
                glyphcount,
                glyphmetrics as _,
                issideways.into(),
            )
        }
    }
    pub unsafe fn GetGlyphIndices(
        &self,
        codepoints: *const u32,
        codepointcount: u32,
        glyphindices: *mut u16,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetGlyphIndices)(
                windows_core::Interface::as_raw(self),
                codepoints,
                codepointcount,
                glyphindices as _,
            )
        }
    }
}
#[repr(C)]
pub struct IDWriteFontFace_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetType: usize,
    GetFiles: usize,
    GetIndex: usize,
    GetSimulations: usize,
    IsSymbolFont: usize,
    pub GetMetrics: unsafe extern "system" fn(*mut core::ffi::c_void, *mut DWRITE_FONT_METRICS),
    GetGlyphCount: usize,
    pub GetDesignGlyphMetrics: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const u16,
        u32,
        *mut DWRITE_GLYPH_METRICS,
        windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub GetGlyphIndices: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const u32,
        u32,
        *mut u16,
    ) -> windows_core::HRESULT,
    TryGetFontTable: usize,
    ReleaseFontTable: usize,
    GetGlyphRunOutline: usize,
    GetRecommendedRenderingMode: usize,
    GetGdiCompatibleMetrics: usize,
    GetGdiCompatibleGlyphMetrics: usize,
}
impl windows_core::RuntimeName for IDWriteFontFace {}
windows_core::imp::define_interface!(
    IDWriteFontFamily,
    IDWriteFontFamily_Vtbl,
    0xda20d8ef_812a_4c43_9802_62ec4abd7add
);
impl core::ops::Deref for IDWriteFontFamily {
    type Target = IDWriteFontList;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IDWriteFontFamily, windows_core::IUnknown, IDWriteFontList);
impl IDWriteFontFamily {
    pub unsafe fn GetFirstMatchingFont(
        &self,
        weight: DWRITE_FONT_WEIGHT,
        stretch: DWRITE_FONT_STRETCH,
        style: DWRITE_FONT_STYLE,
    ) -> windows_core::Result<IDWriteFont> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFirstMatchingFont)(
                windows_core::Interface::as_raw(self),
                weight,
                stretch,
                style,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDWriteFontFamily_Vtbl {
    pub base__: IDWriteFontList_Vtbl,
    GetFamilyNames: usize,
    pub GetFirstMatchingFont: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        DWRITE_FONT_WEIGHT,
        DWRITE_FONT_STRETCH,
        DWRITE_FONT_STYLE,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    GetMatchingFonts: usize,
}
impl windows_core::RuntimeName for IDWriteFontFamily {}
windows_core::imp::define_interface!(
    IDWriteFontList,
    IDWriteFontList_Vtbl,
    0x1a0d8438_1d97_4ec1_aef9_a2fb86ed6acb
);
windows_core::imp::interface_hierarchy!(IDWriteFontList, windows_core::IUnknown);
#[repr(C)]
pub struct IDWriteFontList_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetFontCollection: usize,
    GetFontCount: usize,
    GetFont: usize,
}
impl windows_core::RuntimeName for IDWriteFontList {}
windows_core::imp::define_interface!(
    IDWriteInlineObject,
    IDWriteInlineObject_Vtbl,
    0x8339fde3_106f_47ab_8373_1c6295eb10b3
);
windows_core::imp::interface_hierarchy!(IDWriteInlineObject, windows_core::IUnknown);
#[repr(C)]
pub struct IDWriteInlineObject_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Draw: usize,
    GetMetrics: usize,
    GetOverhangMetrics: usize,
    GetBreakConditions: usize,
}
impl windows_core::RuntimeName for IDWriteInlineObject {}
windows_core::imp::define_interface!(
    IDWriteTextFormat,
    IDWriteTextFormat_Vtbl,
    0x9c906818_31d7_4fd3_a151_7c5e225db55a
);
windows_core::imp::interface_hierarchy!(IDWriteTextFormat, windows_core::IUnknown);
impl IDWriteTextFormat {
    pub unsafe fn SetTrimming<P1>(
        &self,
        trimmingoptions: *const DWRITE_TRIMMING,
        trimmingsign: P1,
    ) -> windows_core::HRESULT
    where
        P1: windows_core::Param<IDWriteInlineObject>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetTrimming)(
                windows_core::Interface::as_raw(self),
                trimmingoptions,
                trimmingsign.param().abi(),
            )
        }
    }
}
#[repr(C)]
pub struct IDWriteTextFormat_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    SetTextAlignment: usize,
    SetParagraphAlignment: usize,
    SetWordWrapping: usize,
    SetReadingDirection: usize,
    SetFlowDirection: usize,
    SetIncrementalTabStop: usize,
    pub SetTrimming: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const DWRITE_TRIMMING,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    SetLineSpacing: usize,
    GetTextAlignment: usize,
    GetParagraphAlignment: usize,
    GetWordWrapping: usize,
    GetReadingDirection: usize,
    GetFlowDirection: usize,
    GetIncrementalTabStop: usize,
    GetTrimming: usize,
    GetLineSpacing: usize,
    GetFontCollection: usize,
    GetFontFamilyNameLength: usize,
    GetFontFamilyName: usize,
    GetFontWeight: usize,
    GetFontStyle: usize,
    GetFontStretch: usize,
    GetFontSize: usize,
    GetLocaleNameLength: usize,
    GetLocaleName: usize,
}
impl windows_core::RuntimeName for IDWriteTextFormat {}
windows_core::imp::define_interface!(
    IExpressionAnimation,
    IExpressionAnimation_Vtbl,
    0x7fff5826_1992_56c0_9060_5ade561a4f2d
);
impl windows_core::RuntimeType for IExpressionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.IExpressionAnimation",
    );
}
#[repr(C)]
pub struct IExpressionAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IInsetClip,
    IInsetClip_Vtbl,
    0xf9d99475_7b59_5b28_a1d2_b832da6988c9
);
impl windows_core::RuntimeType for IInsetClip {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.IInsetClip");
}
#[repr(C)]
pub struct IInsetClip_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IInteractionTracker,
    IInteractionTracker_Vtbl,
    0x02d8ec1f_8f04_505e_bd1e_47b2a204de51
);
impl windows_core::RuntimeType for IInteractionTracker {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.IInteractionTracker",
    );
}
#[repr(C)]
pub struct IInteractionTracker_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub InteractionSources: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    IsPositionRoundingSuggested: usize,
    MaxPosition: usize,
    pub SetMaxPosition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    MaxScale: usize,
    SetMaxScale: usize,
    MinPosition: usize,
    SetMinPosition: usize,
    MinScale: usize,
    SetMinScale: usize,
    NaturalRestingPosition: usize,
    NaturalRestingScale: usize,
    Owner: usize,
    Position: usize,
    PositionInertiaDecayRate: usize,
    SetPositionInertiaDecayRate: usize,
    PositionVelocityInPixelsPerSecond: usize,
    Scale: usize,
    ScaleInertiaDecayRate: usize,
    SetScaleInertiaDecayRate: usize,
    ScaleVelocityInPercentPerSecond: usize,
    AdjustPositionXIfGreaterThanThreshold: usize,
    AdjustPositionYIfGreaterThanThreshold: usize,
    ConfigurePositionXInertiaModifiers: usize,
    ConfigurePositionYInertiaModifiers: usize,
    ConfigureScaleInertiaModifiers: usize,
    pub TryUpdatePosition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector3,
        *mut i32,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IInteractionTrackerCustomAnimationStateEnteredArgs,
    IInteractionTrackerCustomAnimationStateEnteredArgs_Vtbl,
    0x7464035c_cfce_56da_9472_420f276bd0a5
);
impl windows_core::RuntimeType for IInteractionTrackerCustomAnimationStateEnteredArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.IInteractionTrackerCustomAnimationStateEnteredArgs",
    );
}
#[repr(C)]
pub struct IInteractionTrackerCustomAnimationStateEnteredArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IInteractionTrackerIdleStateEnteredArgs,
    IInteractionTrackerIdleStateEnteredArgs_Vtbl,
    0x199094ab_15fd_539c_97b8_964a8196f777
);
impl windows_core::RuntimeType for IInteractionTrackerIdleStateEnteredArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.IInteractionTrackerIdleStateEnteredArgs",
    );
}
#[repr(C)]
pub struct IInteractionTrackerIdleStateEnteredArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IInteractionTrackerInertiaStateEnteredArgs,
    IInteractionTrackerInertiaStateEnteredArgs_Vtbl,
    0x5b76c949_a4d0_5c9d_9292_7013ae9656c7
);
impl windows_core::RuntimeType for IInteractionTrackerInertiaStateEnteredArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.IInteractionTrackerInertiaStateEnteredArgs",
    );
}
#[repr(C)]
pub struct IInteractionTrackerInertiaStateEnteredArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IInteractionTrackerInteractingStateEnteredArgs,
    IInteractionTrackerInteractingStateEnteredArgs_Vtbl,
    0x70d29b84_0931_5f17_a8a1_82f8f8782532
);
impl windows_core::RuntimeType for IInteractionTrackerInteractingStateEnteredArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.IInteractionTrackerInteractingStateEnteredArgs",
    );
}
#[repr(C)]
pub struct IInteractionTrackerInteractingStateEnteredArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IInteractionTrackerOwner,
    IInteractionTrackerOwner_Vtbl,
    0x8869779d_1d2a_5816_836a_68a910507d87
);
impl windows_core::RuntimeType for IInteractionTrackerOwner {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.IInteractionTrackerOwner",
    );
}
windows_core::imp::interface_hierarchy!(
    IInteractionTrackerOwner,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl IInteractionTrackerOwner {
    pub fn CustomAnimationStateEntered<P0, P1>(
        &self,
        sender: P0,
        args: P1,
    ) -> windows_core::Result<()>
    where
        P0: windows_core::Param<InteractionTracker>,
        P1: windows_core::Param<InteractionTrackerCustomAnimationStateEnteredArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CustomAnimationStateEntered)(
                windows_core::Interface::as_raw(self),
                sender.param().abi(),
                args.param().abi(),
            )
            .ok()
        }
    }
    pub fn IdleStateEntered<P0, P1>(&self, sender: P0, args: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<InteractionTracker>,
        P1: windows_core::Param<InteractionTrackerIdleStateEnteredArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).IdleStateEntered)(
                windows_core::Interface::as_raw(self),
                sender.param().abi(),
                args.param().abi(),
            )
            .ok()
        }
    }
    pub fn InertiaStateEntered<P0, P1>(&self, sender: P0, args: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<InteractionTracker>,
        P1: windows_core::Param<InteractionTrackerInertiaStateEnteredArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InertiaStateEntered)(
                windows_core::Interface::as_raw(self),
                sender.param().abi(),
                args.param().abi(),
            )
            .ok()
        }
    }
    pub fn InteractingStateEntered<P0, P1>(&self, sender: P0, args: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<InteractionTracker>,
        P1: windows_core::Param<InteractionTrackerInteractingStateEnteredArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InteractingStateEntered)(
                windows_core::Interface::as_raw(self),
                sender.param().abi(),
                args.param().abi(),
            )
            .ok()
        }
    }
    pub fn RequestIgnored<P0, P1>(&self, sender: P0, args: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<InteractionTracker>,
        P1: windows_core::Param<InteractionTrackerRequestIgnoredArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RequestIgnored)(
                windows_core::Interface::as_raw(self),
                sender.param().abi(),
                args.param().abi(),
            )
            .ok()
        }
    }
    pub fn ValuesChanged<P0, P1>(&self, sender: P0, args: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<InteractionTracker>,
        P1: windows_core::Param<InteractionTrackerValuesChangedArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ValuesChanged)(
                windows_core::Interface::as_raw(self),
                sender.param().abi(),
                args.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeName for IInteractionTrackerOwner {
    const NAME: &'static str = "Microsoft.UI.Composition.Interactions.IInteractionTrackerOwner";
}
pub trait IInteractionTrackerOwner_Impl: windows_core::IUnknownImpl {
    fn CustomAnimationStateEntered(
        &self,
        sender: windows_core::Ref<InteractionTracker>,
        args: windows_core::Ref<InteractionTrackerCustomAnimationStateEnteredArgs>,
    ) -> windows_core::Result<()>;
    fn IdleStateEntered(
        &self,
        sender: windows_core::Ref<InteractionTracker>,
        args: windows_core::Ref<InteractionTrackerIdleStateEnteredArgs>,
    ) -> windows_core::Result<()>;
    fn InertiaStateEntered(
        &self,
        sender: windows_core::Ref<InteractionTracker>,
        args: windows_core::Ref<InteractionTrackerInertiaStateEnteredArgs>,
    ) -> windows_core::Result<()>;
    fn InteractingStateEntered(
        &self,
        sender: windows_core::Ref<InteractionTracker>,
        args: windows_core::Ref<InteractionTrackerInteractingStateEnteredArgs>,
    ) -> windows_core::Result<()>;
    fn RequestIgnored(
        &self,
        sender: windows_core::Ref<InteractionTracker>,
        args: windows_core::Ref<InteractionTrackerRequestIgnoredArgs>,
    ) -> windows_core::Result<()>;
    fn ValuesChanged(
        &self,
        sender: windows_core::Ref<InteractionTracker>,
        args: windows_core::Ref<InteractionTrackerValuesChangedArgs>,
    ) -> windows_core::Result<()>;
}
impl IInteractionTrackerOwner_Vtbl {
    pub const fn new<Identity: IInteractionTrackerOwner_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn CustomAnimationStateEntered<
            Identity: IInteractionTrackerOwner_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            sender: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IInteractionTrackerOwner_Impl::CustomAnimationStateEntered(
                    this,
                    core::mem::transmute_copy(&sender),
                    core::mem::transmute_copy(&args),
                )
                .into()
            }
        }
        unsafe extern "system" fn IdleStateEntered<
            Identity: IInteractionTrackerOwner_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            sender: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IInteractionTrackerOwner_Impl::IdleStateEntered(
                    this,
                    core::mem::transmute_copy(&sender),
                    core::mem::transmute_copy(&args),
                )
                .into()
            }
        }
        unsafe extern "system" fn InertiaStateEntered<
            Identity: IInteractionTrackerOwner_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            sender: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IInteractionTrackerOwner_Impl::InertiaStateEntered(
                    this,
                    core::mem::transmute_copy(&sender),
                    core::mem::transmute_copy(&args),
                )
                .into()
            }
        }
        unsafe extern "system" fn InteractingStateEntered<
            Identity: IInteractionTrackerOwner_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            sender: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IInteractionTrackerOwner_Impl::InteractingStateEntered(
                    this,
                    core::mem::transmute_copy(&sender),
                    core::mem::transmute_copy(&args),
                )
                .into()
            }
        }
        unsafe extern "system" fn RequestIgnored<
            Identity: IInteractionTrackerOwner_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            sender: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IInteractionTrackerOwner_Impl::RequestIgnored(
                    this,
                    core::mem::transmute_copy(&sender),
                    core::mem::transmute_copy(&args),
                )
                .into()
            }
        }
        unsafe extern "system" fn ValuesChanged<
            Identity: IInteractionTrackerOwner_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            sender: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IInteractionTrackerOwner_Impl::ValuesChanged(
                    this,
                    core::mem::transmute_copy(&sender),
                    core::mem::transmute_copy(&args),
                )
                .into()
            }
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<
                Identity,
                IInteractionTrackerOwner,
                OFFSET,
            >(),
            CustomAnimationStateEntered: CustomAnimationStateEntered::<Identity, OFFSET>,
            IdleStateEntered: IdleStateEntered::<Identity, OFFSET>,
            InertiaStateEntered: InertiaStateEntered::<Identity, OFFSET>,
            InteractingStateEntered: InteractingStateEntered::<Identity, OFFSET>,
            RequestIgnored: RequestIgnored::<Identity, OFFSET>,
            ValuesChanged: ValuesChanged::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IInteractionTrackerOwner as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct IInteractionTrackerOwner_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CustomAnimationStateEntered: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub IdleStateEntered: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub InertiaStateEntered: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub InteractingStateEntered: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RequestIgnored: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ValuesChanged: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IInteractionTrackerRequestIgnoredArgs,
    IInteractionTrackerRequestIgnoredArgs_Vtbl,
    0xc276205e_f7a5_5ba2_ad45_d12c3c339149
);
impl windows_core::RuntimeType for IInteractionTrackerRequestIgnoredArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.IInteractionTrackerRequestIgnoredArgs",
    );
}
#[repr(C)]
pub struct IInteractionTrackerRequestIgnoredArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IInteractionTrackerStatics,
    IInteractionTrackerStatics_Vtbl,
    0x7ac9867a_e16e_56ef_9809_f6e404240f50
);
impl windows_core::RuntimeType for IInteractionTrackerStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.IInteractionTrackerStatics",
    );
}
#[repr(C)]
pub struct IInteractionTrackerStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    Create: usize,
    pub CreateWithOwner: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IInteractionTrackerStatics2,
    IInteractionTrackerStatics2_Vtbl,
    0x25658e4c_b99f_5108_aab7_1cc44f11508b
);
impl windows_core::RuntimeType for IInteractionTrackerStatics2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.IInteractionTrackerStatics2",
    );
}
#[repr(C)]
pub struct IInteractionTrackerStatics2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub SetBindingMode: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        InteractionBindingAxisModes,
    ) -> windows_core::HRESULT,
    pub GetBindingMode: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut InteractionBindingAxisModes,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IInteractionTrackerValuesChangedArgs,
    IInteractionTrackerValuesChangedArgs_Vtbl,
    0x9b495bed_1cf7_55c1_82b9_8022cbf3c766
);
impl windows_core::RuntimeType for IInteractionTrackerValuesChangedArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.IInteractionTrackerValuesChangedArgs",
    );
}
#[repr(C)]
pub struct IInteractionTrackerValuesChangedArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Position: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ISequentialStream,
    ISequentialStream_Vtbl,
    0x0c733a30_2a1c_11ce_ade5_00aa0044773d
);
windows_core::imp::interface_hierarchy!(ISequentialStream, windows_core::IUnknown);
#[repr(C)]
pub struct ISequentialStream_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Read: usize,
    Write: usize,
}
impl windows_core::RuntimeName for ISequentialStream {}
windows_core::imp::define_interface!(
    ISpriteVisual,
    ISpriteVisual_Vtbl,
    0x7e964632_45e4_5761_806d_5b4022c14f26
);
impl windows_core::RuntimeType for ISpriteVisual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.ISpriteVisual");
}
#[repr(C)]
pub struct ISpriteVisual_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    Brush: usize,
    pub SetBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IStream,
    IStream_Vtbl,
    0x0000000c_0000_0000_c000_000000000046
);
impl core::ops::Deref for IStream {
    type Target = ISequentialStream;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IStream, windows_core::IUnknown, ISequentialStream);
#[repr(C)]
pub struct IStream_Vtbl {
    pub base__: ISequentialStream_Vtbl,
    Seek: usize,
    SetSize: usize,
    CopyTo: usize,
    Commit: usize,
    Revert: usize,
    LockRegion: usize,
    UnlockRegion: usize,
    Stat: usize,
    Clone: usize,
}
impl windows_core::RuntimeName for IStream {}
windows_core::imp::define_interface!(
    IVisual,
    IVisual_Vtbl,
    0xc0eeab6c_c897_5ac6_a1c9_63abd5055b9b
);
impl windows_core::RuntimeType for IVisual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.IVisual");
}
#[repr(C)]
pub struct IVisual_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    AnchorPoint: usize,
    SetAnchorPoint: usize,
    BackfaceVisibility: usize,
    SetBackfaceVisibility: usize,
    BorderMode: usize,
    SetBorderMode: usize,
    CenterPoint: usize,
    SetCenterPoint: usize,
    Clip: usize,
    pub SetClip: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    CompositeMode: usize,
    SetCompositeMode: usize,
    IsVisible: usize,
    pub SetIsVisible:
        unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    Offset: usize,
    pub SetOffset: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    Opacity: usize,
    SetOpacity: usize,
    Orientation: usize,
    SetOrientation: usize,
    Parent: usize,
    RotationAngle: usize,
    SetRotationAngle: usize,
    RotationAngleInDegrees: usize,
    SetRotationAngleInDegrees: usize,
    RotationAxis: usize,
    SetRotationAxis: usize,
    Scale: usize,
    SetScale: usize,
    Size: usize,
    pub SetSize: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IVisualCollection,
    IVisualCollection_Vtbl,
    0xd002896d_67d8_5f69_ab70_581fa3bf370f
);
impl windows_core::RuntimeType for IVisualCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.IVisualCollection");
}
#[repr(C)]
pub struct IVisualCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    Count: usize,
    InsertAbove: usize,
    pub InsertAtBottom: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub InsertAtTop: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    InsertBelow: usize,
    pub Remove: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IVisualElement,
    IVisualElement_Vtbl,
    0x2180f1f5_b5d8_4bf6_920a_12006e63efef
);
impl windows_core::RuntimeType for IVisualElement {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Composition.IVisualElement");
}
windows_core::imp::interface_hierarchy!(
    IVisualElement,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for IVisualElement {
    const NAME: &'static str = "Microsoft.UI.Composition.IVisualElement";
}
pub trait IVisualElement_Impl: windows_core::IUnknownImpl {}
impl IVisualElement_Vtbl {
    pub const fn new<Identity: IVisualElement_Impl, const OFFSET: isize>() -> Self {
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, IVisualElement, OFFSET>(),
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IVisualElement as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct IVisualElement_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IVisualInteractionSource,
    IVisualInteractionSource_Vtbl,
    0xea595c95_b9cb_5cd4_bb9c_4934ff329063
);
impl windows_core::RuntimeType for IVisualInteractionSource {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.IVisualInteractionSource",
    );
}
#[repr(C)]
pub struct IVisualInteractionSource_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    IsPositionXRailsEnabled: usize,
    SetIsPositionXRailsEnabled: usize,
    IsPositionYRailsEnabled: usize,
    SetIsPositionYRailsEnabled: usize,
    ManipulationRedirectionMode: usize,
    pub SetManipulationRedirectionMode: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VisualInteractionSourceRedirectionMode,
    ) -> windows_core::HRESULT,
    PositionXChainingMode: usize,
    SetPositionXChainingMode: usize,
    PositionXSourceMode: usize,
    SetPositionXSourceMode: usize,
    PositionYChainingMode: usize,
    SetPositionYChainingMode: usize,
    PositionYSourceMode: usize,
    pub SetPositionYSourceMode: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        InteractionSourceMode,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IVisualInteractionSourceStatics,
    IVisualInteractionSourceStatics_Vtbl,
    0x5fc9c763_e2e5_530e_87cd_b93118ade8a3
);
impl windows_core::RuntimeType for IVisualInteractionSourceStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.IVisualInteractionSourceStatics",
    );
}
#[repr(C)]
pub struct IVisualInteractionSourceStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Create: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IVisualInteractionSourceStatics2,
    IVisualInteractionSourceStatics2_Vtbl,
    0xa6b494fe_12a1_5a73_b87e_4c4ef58eac6c
);
impl windows_core::RuntimeType for IVisualInteractionSourceStatics2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.IVisualInteractionSourceStatics2",
    );
}
#[repr(C)]
pub struct IVisualInteractionSourceStatics2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateFromIVisualElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsetClip(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    InsetClip,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(InsetClip, CompositionClip, CompositionObject);
impl InsetClip {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for InsetClip {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IInsetClip>();
}
unsafe impl windows_core::Interface for InsetClip {
    type Vtable = <IInsetClip as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IInsetClip as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for InsetClip {
    const NAME: &'static str = "Microsoft.UI.Composition.InsetClip";
}
unsafe impl Send for InsetClip {}
unsafe impl Sync for InsetClip {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InteractionBindingAxisModes(pub u32);
impl InteractionBindingAxisModes {
    pub const None: Self = Self(0);
    pub const PositionX: Self = Self(1);
    pub const PositionY: Self = Self(2);
    pub const Scale: Self = Self(4);
}
impl windows_core::imp::TypeKind for InteractionBindingAxisModes {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for InteractionBindingAxisModes {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Microsoft.UI.Composition.Interactions.InteractionBindingAxisModes;u4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.InteractionBindingAxisModes",
    );
}
impl InteractionBindingAxisModes {
    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
impl core::ops::BitOr for InteractionBindingAxisModes {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl core::ops::BitAnd for InteractionBindingAxisModes {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}
impl core::ops::BitOrAssign for InteractionBindingAxisModes {
    fn bitor_assign(&mut self, other: Self) {
        self.0.bitor_assign(other.0);
    }
}
impl core::ops::BitAndAssign for InteractionBindingAxisModes {
    fn bitand_assign(&mut self, other: Self) {
        self.0.bitand_assign(other.0);
    }
}
impl core::ops::Not for InteractionBindingAxisModes {
    type Output = Self;
    fn not(self) -> Self {
        Self(self.0.not())
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InteractionSourceMode(pub i32);
impl InteractionSourceMode {
    pub const Disabled: Self = Self(0);
    pub const EnabledWithInertia: Self = Self(1);
    pub const EnabledWithoutInertia: Self = Self(2);
}
impl windows_core::imp::TypeKind for InteractionSourceMode {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for InteractionSourceMode {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Microsoft.UI.Composition.Interactions.InteractionSourceMode;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.InteractionSourceMode",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionTracker(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    InteractionTracker,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(InteractionTracker, CompositionObject);
impl InteractionTracker {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub fn InteractionSources(
        &self,
    ) -> windows_core::Result<CompositionInteractionSourceCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).InteractionSources)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetMaxPosition(&self, value: windows_numerics::Vector3) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetMaxPosition)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn TryUpdatePosition(&self, value: windows_numerics::Vector3) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryUpdatePosition)(
                windows_core::Interface::as_raw(self),
                value,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn CreateWithOwner<P0, P1>(compositor: P0, owner: P1) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<Compositor>,
        P1: windows_core::Param<IInteractionTrackerOwner>,
    {
        Self::IInteractionTrackerStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateWithOwner)(
                windows_core::Interface::as_raw(this),
                compositor.param().abi(),
                owner.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn SetBindingMode<P0, P1>(
        boundtracker1: P0,
        boundtracker2: P1,
        axismode: InteractionBindingAxisModes,
    ) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Self>,
        P1: windows_core::Param<Self>,
    {
        Self::IInteractionTrackerStatics2(|this| unsafe {
            (windows_core::Interface::vtable(this).SetBindingMode)(
                windows_core::Interface::as_raw(this),
                boundtracker1.param().abi(),
                boundtracker2.param().abi(),
                axismode,
            )
            .ok()
        })
    }
    pub fn GetBindingMode<P0, P1>(
        boundtracker1: P0,
        boundtracker2: P1,
    ) -> windows_core::Result<InteractionBindingAxisModes>
    where
        P0: windows_core::Param<Self>,
        P1: windows_core::Param<Self>,
    {
        Self::IInteractionTrackerStatics2(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetBindingMode)(
                windows_core::Interface::as_raw(this),
                boundtracker1.param().abi(),
                boundtracker2.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    fn IInteractionTrackerStatics<
        R,
        F: FnOnce(&IInteractionTrackerStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            InteractionTracker,
            IInteractionTrackerStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    fn IInteractionTrackerStatics2<
        R,
        F: FnOnce(&IInteractionTrackerStatics2) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            InteractionTracker,
            IInteractionTrackerStatics2,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for InteractionTracker {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IInteractionTracker>();
}
unsafe impl windows_core::Interface for InteractionTracker {
    type Vtable = <IInteractionTracker as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IInteractionTracker as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for InteractionTracker {
    const NAME: &'static str = "Microsoft.UI.Composition.Interactions.InteractionTracker";
}
unsafe impl Send for InteractionTracker {}
unsafe impl Sync for InteractionTracker {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionTrackerCustomAnimationStateEnteredArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    InteractionTrackerCustomAnimationStateEnteredArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for InteractionTrackerCustomAnimationStateEnteredArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<
        Self,
        IInteractionTrackerCustomAnimationStateEnteredArgs,
    >();
}
unsafe impl windows_core::Interface for InteractionTrackerCustomAnimationStateEnteredArgs {
    type Vtable =
        <IInteractionTrackerCustomAnimationStateEnteredArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IInteractionTrackerCustomAnimationStateEnteredArgs as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for InteractionTrackerCustomAnimationStateEnteredArgs {
    const NAME: &'static str =
        "Microsoft.UI.Composition.Interactions.InteractionTrackerCustomAnimationStateEnteredArgs";
}
unsafe impl Send for InteractionTrackerCustomAnimationStateEnteredArgs {}
unsafe impl Sync for InteractionTrackerCustomAnimationStateEnteredArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionTrackerIdleStateEnteredArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    InteractionTrackerIdleStateEnteredArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for InteractionTrackerIdleStateEnteredArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IInteractionTrackerIdleStateEnteredArgs>(
        );
}
unsafe impl windows_core::Interface for InteractionTrackerIdleStateEnteredArgs {
    type Vtable = <IInteractionTrackerIdleStateEnteredArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IInteractionTrackerIdleStateEnteredArgs as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for InteractionTrackerIdleStateEnteredArgs {
    const NAME: &'static str =
        "Microsoft.UI.Composition.Interactions.InteractionTrackerIdleStateEnteredArgs";
}
unsafe impl Send for InteractionTrackerIdleStateEnteredArgs {}
unsafe impl Sync for InteractionTrackerIdleStateEnteredArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionTrackerInertiaStateEnteredArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    InteractionTrackerInertiaStateEnteredArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for InteractionTrackerInertiaStateEnteredArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<
        Self,
        IInteractionTrackerInertiaStateEnteredArgs,
    >();
}
unsafe impl windows_core::Interface for InteractionTrackerInertiaStateEnteredArgs {
    type Vtable = <IInteractionTrackerInertiaStateEnteredArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IInteractionTrackerInertiaStateEnteredArgs as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for InteractionTrackerInertiaStateEnteredArgs {
    const NAME: &'static str =
        "Microsoft.UI.Composition.Interactions.InteractionTrackerInertiaStateEnteredArgs";
}
unsafe impl Send for InteractionTrackerInertiaStateEnteredArgs {}
unsafe impl Sync for InteractionTrackerInertiaStateEnteredArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionTrackerInteractingStateEnteredArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    InteractionTrackerInteractingStateEnteredArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for InteractionTrackerInteractingStateEnteredArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<
        Self,
        IInteractionTrackerInteractingStateEnteredArgs,
    >();
}
unsafe impl windows_core::Interface for InteractionTrackerInteractingStateEnteredArgs {
    type Vtable =
        <IInteractionTrackerInteractingStateEnteredArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IInteractionTrackerInteractingStateEnteredArgs as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for InteractionTrackerInteractingStateEnteredArgs {
    const NAME: &'static str =
        "Microsoft.UI.Composition.Interactions.InteractionTrackerInteractingStateEnteredArgs";
}
unsafe impl Send for InteractionTrackerInteractingStateEnteredArgs {}
unsafe impl Sync for InteractionTrackerInteractingStateEnteredArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionTrackerRequestIgnoredArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    InteractionTrackerRequestIgnoredArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for InteractionTrackerRequestIgnoredArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IInteractionTrackerRequestIgnoredArgs>();
}
unsafe impl windows_core::Interface for InteractionTrackerRequestIgnoredArgs {
    type Vtable = <IInteractionTrackerRequestIgnoredArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IInteractionTrackerRequestIgnoredArgs as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for InteractionTrackerRequestIgnoredArgs {
    const NAME: &'static str =
        "Microsoft.UI.Composition.Interactions.InteractionTrackerRequestIgnoredArgs";
}
unsafe impl Send for InteractionTrackerRequestIgnoredArgs {}
unsafe impl Sync for InteractionTrackerRequestIgnoredArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionTrackerValuesChangedArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    InteractionTrackerValuesChangedArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl InteractionTrackerValuesChangedArgs {
    pub fn Position(&self) -> windows_core::Result<windows_numerics::Vector3> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Position)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
impl windows_core::RuntimeType for InteractionTrackerValuesChangedArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IInteractionTrackerValuesChangedArgs>();
}
unsafe impl windows_core::Interface for InteractionTrackerValuesChangedArgs {
    type Vtable = <IInteractionTrackerValuesChangedArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IInteractionTrackerValuesChangedArgs as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for InteractionTrackerValuesChangedArgs {
    const NAME: &'static str =
        "Microsoft.UI.Composition.Interactions.InteractionTrackerValuesChangedArgs";
}
unsafe impl Send for InteractionTrackerValuesChangedArgs {}
unsafe impl Sync for InteractionTrackerValuesChangedArgs {}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub Width: f32,
    pub Height: f32,
}
impl windows_core::imp::TypeKind for Size {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Size {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.Foundation.Size;f4;f4)");
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Foundation.Size");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpriteVisual(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    SpriteVisual,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(SpriteVisual, ContainerVisual, Visual, CompositionObject);
impl SpriteVisual {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub fn Children(&self) -> windows_core::Result<VisualCollection> {
        let this = &windows_core::Interface::cast::<IContainerVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Children)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetBrush<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<CompositionBrush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetBrush)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn SetClip<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<CompositionClip>,
    {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetClip)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn SetIsVisible(&self, value: bool) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIsVisible)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub fn SetOffset(&self, value: windows_numerics::Vector3) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOffset)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub fn SetSize(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetSize)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for SpriteVisual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISpriteVisual>();
}
unsafe impl windows_core::Interface for SpriteVisual {
    type Vtable = <ISpriteVisual as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ISpriteVisual as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for SpriteVisual {
    const NAME: &'static str = "Microsoft.UI.Composition.SpriteVisual";
}
unsafe impl Send for SpriteVisual {}
unsafe impl Sync for SpriteVisual {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Visual(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Visual, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Visual, CompositionObject);
impl Visual {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub fn SetClip<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<CompositionClip>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetClip)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn SetIsVisible(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsVisible)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn SetOffset(&self, value: windows_numerics::Vector3) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetOffset)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn SetSize(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetSize)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for Visual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVisual>();
}
unsafe impl windows_core::Interface for Visual {
    type Vtable = <IVisual as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVisual as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for Visual {
    const NAME: &'static str = "Microsoft.UI.Composition.Visual";
}
unsafe impl Send for Visual {}
unsafe impl Sync for Visual {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisualCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    VisualCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(VisualCollection, CompositionObject);
impl VisualCollection {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub fn InsertAtBottom<P0>(&self, newchild: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Visual>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InsertAtBottom)(
                windows_core::Interface::as_raw(self),
                newchild.param().abi(),
            )
            .ok()
        }
    }
    pub fn InsertAtTop<P0>(&self, newchild: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Visual>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InsertAtTop)(
                windows_core::Interface::as_raw(self),
                newchild.param().abi(),
            )
            .ok()
        }
    }
    pub fn Remove<P0>(&self, child: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Visual>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Remove)(
                windows_core::Interface::as_raw(self),
                child.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for VisualCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVisualCollection>();
}
unsafe impl windows_core::Interface for VisualCollection {
    type Vtable = <IVisualCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVisualCollection as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for VisualCollection {
    const NAME: &'static str = "Microsoft.UI.Composition.VisualCollection";
}
unsafe impl Send for VisualCollection {}
unsafe impl Sync for VisualCollection {}
impl IntoIterator for VisualCollection {
    type Item = Visual;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(&self)
    }
}
impl IntoIterator for &VisualCollection {
    type Item = Visual;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        windows_collections::BufferedIterator::new(self.First().unwrap())
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisualInteractionSource(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    VisualInteractionSource,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(VisualInteractionSource, CompositionObject);
impl VisualInteractionSource {
    pub fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub fn SetManipulationRedirectionMode(
        &self,
        value: VisualInteractionSourceRedirectionMode,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetManipulationRedirectionMode)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn SetPositionYSourceMode(&self, value: InteractionSourceMode) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetPositionYSourceMode)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn Create<P0>(source: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<Visual>,
    {
        Self::IVisualInteractionSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Create)(
                windows_core::Interface::as_raw(this),
                source.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateFromIVisualElement<P0>(source: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<IVisualElement>,
    {
        Self::IVisualInteractionSourceStatics2(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromIVisualElement)(
                windows_core::Interface::as_raw(this),
                source.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IVisualInteractionSourceStatics<
        R,
        F: FnOnce(&IVisualInteractionSourceStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            VisualInteractionSource,
            IVisualInteractionSourceStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    fn IVisualInteractionSourceStatics2<
        R,
        F: FnOnce(&IVisualInteractionSourceStatics2) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            VisualInteractionSource,
            IVisualInteractionSourceStatics2,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for VisualInteractionSource {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVisualInteractionSource>();
}
unsafe impl windows_core::Interface for VisualInteractionSource {
    type Vtable = <IVisualInteractionSource as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVisualInteractionSource as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for VisualInteractionSource {
    const NAME: &'static str = "Microsoft.UI.Composition.Interactions.VisualInteractionSource";
}
unsafe impl Send for VisualInteractionSource {}
unsafe impl Sync for VisualInteractionSource {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VisualInteractionSourceRedirectionMode(pub i32);
impl VisualInteractionSourceRedirectionMode {
    pub const Off: Self = Self(0);
    pub const CapableTouchpadOnly: Self = Self(1);
    pub const PointerWheelOnly: Self = Self(2);
    pub const CapableTouchpadAndPointerWheel: Self = Self(3);
}
impl windows_core::imp::TypeKind for VisualInteractionSourceRedirectionMode {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for VisualInteractionSourceRedirectionMode {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Microsoft.UI.Composition.Interactions.VisualInteractionSourceRedirectionMode;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Composition.Interactions.VisualInteractionSourceRedirectionMode",
    );
}
