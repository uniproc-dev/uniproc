#[cfg(native_probe)]
#[allow(
    clippy::all,
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(native_probe)]
pub mod probe {
    use crate::bindings as c;
    use std::cell::RefCell;
    use windows_core::{Interface, Result};

    thread_local! {
        static ROOT: RefCell<Option<c::UIElement>> = const { RefCell::new(None) };
        static EVENTS: RefCell<Vec<windows_core::EventRevoker>> = const { RefCell::new(Vec::new()) };
        static CAPTURES: RefCell<Vec<Capture>> = const { RefCell::new(Vec::new()) };
    }

    struct Capture {
        bitmap: c::RenderTargetBitmap,
        rendered: windows_future::IAsyncAction,
        pixels: Option<windows_future::IAsyncOperation<c::IBuffer>>,
        index: usize,
        saved: bool,
    }

    pub fn capture() {
        let result = ROOT.with_borrow(|root| -> Result<()> {
            let Some(root) = root else {
                eprintln!("CAPTURE no root");
                return Ok(());
            };
            let mut nodes = Vec::new();
            children(&root.cast()?, &mut nodes)?;
            for (index, node) in nodes
                .into_iter()
                .filter(|node| node.cast::<c::Image>().is_ok())
                .enumerate()
            {
                let image = node.cast::<c::Image>()?;
                let element = image.cast::<c::FrameworkElement>()?;
                let source = image.Source()?.cast::<c::SvgImageSource>()?;
                eprintln!(
                    "CAPTURE {index} actual={}x{} raster={}x{} scale={}",
                    element.ActualWidth()?,
                    element.ActualHeight()?,
                    source.RasterizePixelWidth()?,
                    source.RasterizePixelHeight()?,
                    image
                        .cast::<c::UIElement>()?
                        .XamlRoot()?
                        .RasterizationScale()?
                );
                let bitmap: c::RenderTargetBitmap = windows_core::factory::<
                    c::RenderTargetBitmap,
                    windows_core::imp::IGenericFactory,
                >()?
                .ActivateInstance()?;
                let rendered = bitmap.RenderAsync(&image.cast::<c::UIElement>()?)?;
                CAPTURES.with_borrow_mut(|captures| {
                    captures.push(Capture {
                        bitmap,
                        rendered,
                        pixels: None,
                        index,
                        saved: false,
                    })
                });
            }
            Ok(())
        });
        if let Err(error) = result {
            eprintln!("CAPTURE error {error}");
        }
    }

    pub fn poll_captures() -> bool {
        CAPTURES.with_borrow_mut(|captures| {
            for capture in captures.iter_mut().filter(|capture| !capture.saved) {
                let result = (|| -> Result<()> {
                    if capture.rendered.Status()? != windows_future::AsyncStatus::Completed {
                        return Ok(());
                    }
                    if capture.pixels.is_none() {
                        capture.pixels = Some(capture.bitmap.GetPixelsAsync()?);
                    }
                    let pixels = capture.pixels.as_ref().unwrap();
                    if pixels.Status()? != windows_future::AsyncStatus::Completed {
                        return Ok(());
                    }
                    let buffer = pixels.GetResults()?;
                    let width = capture.bitmap.PixelWidth()? as u32;
                    let height = capture.bitmap.PixelHeight()? as u32;
                    let mut bytes = vec![0; width as usize * height as usize * 4];
                    c::DataReader::FromBuffer(&buffer)?.ReadBytes(&mut bytes)?;
                    for pixel in bytes.as_chunks_mut::<4>().0 {
                        pixel.swap(0, 2);
                        if pixel[3] != 0 {
                            let alpha = u16::from(pixel[3]);
                            for channel in &mut pixel[..3] {
                                *channel = (u16::from(*channel) * 255 / alpha).min(255) as u8;
                            }
                        }
                    }
                    let file = format!("image-{}-{}x{}.png", capture.index, width, height);
                    image::save_buffer(&file, &bytes, width, height, image::ColorType::Rgba8)
                        .unwrap();
                    eprintln!("SAVED {file}");
                    capture.saved = true;
                    Ok(())
                })();
                if let Err(error) = result {
                    eprintln!("CAPTURE poll {error}");
                    capture.saved = true;
                }
            }
            !captures.is_empty() && captures.iter().all(|capture| capture.saved)
        })
    }

    fn children(node: &c::DependencyObject, nodes: &mut Vec<c::DependencyObject>) -> Result<()> {
        nodes.push(node.clone());
        for index in 0..c::VisualTreeHelper::GetChildrenCount(node)? {
            children(&c::VisualTreeHelper::GetChild(node, index)?, nodes)?;
        }
        Ok(())
    }

    pub fn install() -> windows_core::EventRevoker {
        c::FocusManager::GotFocus(|_, args| {
            let result = (|| -> Result<()> {
                let element = args.ok()?.NewFocusedElement()?.cast::<c::UIElement>()?;
                let root = element.XamlRoot()?.Content()?;
                ROOT.with_borrow_mut(|slot| *slot = Some(root));
                Ok(())
            })();
            if let Err(error) = result {
                eprintln!("PROBE focus {error}");
            }
        })
        .unwrap()
    }

    pub fn scroll(step: u32) {
        let result = ROOT.with_borrow(|root| -> Result<()> {
            let Some(root) = root else {
                eprintln!("PROBE root not available");
                return Ok(());
            };
            let mut nodes = Vec::new();
            children(&root.cast()?, &mut nodes)?;
            for node in nodes {
                let Ok(viewer) = node.cast::<c::ScrollViewer>() else {
                    continue;
                };
                if step == 1 {
                    let copy = viewer.clone();
                    let revoker = viewer.ViewChanged(move |_, args| {
                        eprintln!(
                            "OFFSET t={:?} y={:?} extent={:?} viewport={:?} intermediate={:?}",
                            std::time::SystemTime::now(),
                            copy.VerticalOffset(),
                            copy.ExtentHeight(),
                            copy.ViewportHeight(),
                            args.ok().and_then(|args| args.IsIntermediate())
                        );
                        if args
                            .ok()
                            .and_then(|args| args.IsIntermediate())
                            .is_ok_and(|intermediate| !intermediate)
                            && let Ok(root) = copy
                                .cast::<c::UIElement>()
                                .and_then(|element| element.XamlRoot())
                                .and_then(|root| root.Content())
                        {
                            let _ = positions(&root, &copy);
                        }
                    })?;
                    EVENTS.with_borrow_mut(|events| events.push(revoker));
                }
                let stride = std::env::var("SCROLL_STRIDE")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(192.0);
                let target = (f64::from(step) * stride)
                    .min(viewer.ExtentHeight()? - viewer.ViewportHeight()?);
                eprintln!(
                    "REQUEST step={step} target={target} current={}",
                    viewer.VerticalOffset()?
                );
                let _ = viewer.ChangeView(None, Some(target), None)?;
                break;
            }
            Ok(())
        });
        if let Err(error) = result {
            eprintln!("PROBE scroll {error}");
        }
    }

    fn positions(root: &c::UIElement, viewer: &c::ScrollViewer) -> Result<()> {
        let mut nodes = Vec::new();
        children(&root.cast()?, &mut nodes)?;
        let offset = viewer.VerticalOffset()?;
        let relative = viewer.cast::<c::UIElement>()?;
        for node in nodes {
            let id = c::AutomationProperties::GetAutomationId(&node)?;
            let Some(index) = id
                .to_string()
                .strip_prefix("row-")
                .and_then(|index| index.parse::<usize>().ok())
            else {
                continue;
            };
            let element = node.cast::<c::UIElement>()?;
            let point = element
                .TransformToVisual(&relative)?
                .TransformPoint(c::Point { x: 0.0, y: 0.0 })?;
            let height = node.cast::<c::FrameworkElement>()?.ActualHeight()?;
            if point.y >= -32.0 && f64::from(point.y) < viewer.ViewportHeight()? {
                let error = f64::from(point.y) - (index as f64 * 32.0 - offset);
                if error.abs() > 1.0 {
                    eprintln!(
                        "ROW_ERROR index={index} y={} height={height} offset={offset} error={error}",
                        point.y
                    );
                }
            }
        }
        Ok(())
    }
}
