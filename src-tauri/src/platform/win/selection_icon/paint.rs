//! Draws the icon image into the layered window at the monitor's DPI.

use windows::Win32::Foundation::{COLORREF, HWND, POINT, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject,
    AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS,
};
use windows::Win32::UI::WindowsAndMessaging::{UpdateLayeredWindow, ULW_ALPHA};

/// Pushes per-pixel-alpha pixels to the layered window.
pub unsafe fn paint(hwnd: HWND, bgra: &[u8], size: i32, pos: POINT) -> bool {
    let screen = GetDC(None);
    let memory = CreateCompatibleDC(Some(screen));
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size,
            biHeight: -size,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits = std::ptr::null_mut();
    let ok = match CreateDIBSection(Some(memory), &info, DIB_RGB_COLORS, &mut bits, None, 0) {
        Ok(bitmap) => {
            std::ptr::copy_nonoverlapping(bgra.as_ptr(), bits as *mut u8, bgra.len());
            let previous = SelectObject(memory, bitmap.into());
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let result = UpdateLayeredWindow(
                hwnd,
                Some(screen),
                Some(&pos),
                Some(&SIZE { cx: size, cy: size }),
                Some(memory),
                Some(&POINT::default()),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );
            SelectObject(memory, previous);
            let _ = DeleteObject(bitmap.into());
            result.is_ok()
        }
        Err(_) => false,
    };
    let _ = DeleteDC(memory);
    ReleaseDC(None, screen);
    ok
}

/// Box-filter resample of straight RGBA into premultiplied BGRA, as layered windows expect.
pub fn resample_premultiplied_bgra(rgba: &[u8], width: u32, height: u32, size: u32) -> Vec<u8> {
    let mut out = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        let y0 = y * height / size;
        let y1 = ((y + 1) * height / size).max(y0 + 1);
        for x in 0..size {
            let x0 = x * width / size;
            let x1 = ((x + 1) * width / size).max(x0 + 1);
            let (mut r, mut g, mut b, mut a, mut n) = (0u32, 0u32, 0u32, 0u32, 0u32);
            for sy in y0..y1 {
                for sx in x0..x1 {
                    let i = ((sy * width + sx) * 4) as usize;
                    let alpha = rgba[i + 3] as u32;
                    r += rgba[i] as u32 * alpha / 255;
                    g += rgba[i + 1] as u32 * alpha / 255;
                    b += rgba[i + 2] as u32 * alpha / 255;
                    a += alpha;
                    n += 1;
                }
            }
            let o = ((y * size + x) * 4) as usize;
            out[o] = (b / n) as u8;
            out[o + 1] = (g / n) as u8;
            out[o + 2] = (r / n) as u8;
            out[o + 3] = (a / n) as u8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_premultiplies_and_swaps_channels() {
        // 2x2 half-transparent red, shrunk to 1x1.
        let rgba = [255, 0, 0, 128].repeat(4);
        assert_eq!(
            resample_premultiplied_bgra(&rgba, 2, 2, 1),
            vec![0, 0, 128, 128]
        );
    }

    #[test]
    fn resample_upscales_by_repeating_pixels() {
        let opaque_blue = [0, 0, 255, 255];
        assert_eq!(
            resample_premultiplied_bgra(&opaque_blue, 1, 1, 2),
            [255, 0, 0, 255].repeat(4)
        );
    }
}
