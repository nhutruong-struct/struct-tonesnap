//! Screen capture plus on-device OCR (Windows.Media.Ocr). The image never leaves the machine;
//! only the recognized text is used, after redaction.

use windows::core::HSTRING;
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, SetStretchBltMode, StretchBlt, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    CAPTUREBLT, DIB_RGB_COLORS, HALFTONE, SRCCOPY,
};

use super::com::on_mta_thread;
use crate::geometry::Rect;

/// Small UI text recognizes much better when upscaled.
const UPSCALE: i32 = 2;

/// Recognized lines of text inside `rect` (physical screen pixels), top to bottom.
pub fn recognize(rect: Rect) -> anyhow::Result<Vec<String>> {
    // Errors become plain text on the COM thread: a `windows` error holds a COM error object
    // that must not be released after that thread uninitializes COM.
    // SAFETY: `recognize_inner` needs COM, which `on_mta_thread` initializes; it releases every
    // GDI object it creates.
    on_mta_thread(move || unsafe { recognize_inner(rect) }.map_err(|e| anyhow::anyhow!("{e}")))
        .ok_or_else(|| anyhow::anyhow!("ocr thread panicked"))?
}

unsafe fn recognize_inner(rect: Rect) -> anyhow::Result<Vec<String>> {
    let max = OcrEngine::MaxImageDimension()? as i32;
    let (w, h) = (rect.width(), rect.height());
    anyhow::ensure!(w > 0 && h > 0, "empty capture rectangle");
    let scale = if w * UPSCALE <= max && h * UPSCALE <= max {
        UPSCALE
    } else {
        1
    };
    let (pixels, out_w, out_h) = capture(rect, scale);
    recognize_pixels(&pixels, out_w, out_h)
}

/// Recognized lines of text in a top-down BGRA image. Runs on a COM thread.
fn recognize_pixels(pixels: &[u8], width: i32, height: i32) -> anyhow::Result<Vec<String>> {
    let engine =
        OcrEngine::TryCreateFromLanguage(&Language::CreateLanguage(&HSTRING::from("en-US"))?)
            .or_else(|_| OcrEngine::TryCreateFromUserProfileLanguages())?;
    let writer = DataWriter::new()?;
    writer.WriteBytes(pixels)?;
    let buffer = writer.DetachBuffer()?;
    let bitmap = SoftwareBitmap::CreateCopyWithAlphaFromBuffer(
        &buffer,
        BitmapPixelFormat::Bgra8,
        width,
        height,
        BitmapAlphaMode::Ignore,
    )?;
    let result = engine.RecognizeAsync(&bitmap)?.join()?;
    let mut lines = Vec::new();
    for line in result.Lines()? {
        let text = line.Text()?.to_string();
        if !text.trim().is_empty() {
            lines.push(text);
        }
    }
    Ok(lines)
}

/// BGRA pixels of a screen rectangle, scaled by `scale`.
unsafe fn capture(rect: Rect, scale: i32) -> (Vec<u8>, i32, i32) {
    let (w, h) = (rect.width(), rect.height());
    let (out_w, out_h) = (w * scale, h * scale);

    let screen = GetDC(None);
    let memory = CreateCompatibleDC(Some(screen));
    let bitmap = CreateCompatibleBitmap(screen, out_w, out_h);
    let previous = SelectObject(memory, bitmap.into());
    SetStretchBltMode(memory, HALFTONE);
    let _ = StretchBlt(
        memory,
        0,
        0,
        out_w,
        out_h,
        Some(screen),
        rect.left,
        rect.top,
        w,
        h,
        SRCCOPY | CAPTUREBLT,
    );

    let mut info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: out_w,
            biHeight: -out_h, // top-down rows
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut pixels = vec![0u8; (out_w * out_h * 4) as usize];
    GetDIBits(
        memory,
        bitmap,
        0,
        out_h as u32,
        Some(pixels.as_mut_ptr() as *mut _),
        &mut info,
        DIB_RGB_COLORS,
    );

    SelectObject(memory, previous);
    let _ = DeleteObject(bitmap.into());
    let _ = DeleteDC(memory);
    ReleaseDC(None, screen);

    // GDI leaves alpha at zero.
    for px in pixels.as_chunks_mut::<4>().0 {
        px[3] = 255;
    }
    (pixels, out_w, out_h)
}

#[cfg(test)]
mod integration_tests {
    use windows::core::w;
    use windows::Win32::Foundation::{COLORREF, RECT};
    use windows::Win32::Graphics::Gdi::{
        CreateFontW, FillRect, GetStockObject, SetBkMode, SetTextColor, TextOutW,
        CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, FW_NORMAL, HBRUSH,
        OUT_DEFAULT_PRECIS, TRANSPARENT, WHITE_BRUSH,
    };

    use super::*;

    /// `text` drawn in black on white, as top-down BGRA pixels.
    unsafe fn render(text: &str, width: i32, height: i32) -> Vec<u8> {
        let screen = GetDC(None);
        let memory = CreateCompatibleDC(Some(screen));
        let bitmap = CreateCompatibleBitmap(screen, width, height);
        let previous = SelectObject(memory, bitmap.into());
        let area = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
        FillRect(memory, &area, HBRUSH(GetStockObject(WHITE_BRUSH).0));
        let font = CreateFontW(
            40,
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            0,
            w!("Segoe UI"),
        );
        let previous_font = SelectObject(memory, font.into());
        SetBkMode(memory, TRANSPARENT);
        SetTextColor(memory, COLORREF(0));
        let wide: Vec<u16> = text.encode_utf16().collect();
        let _ = TextOutW(memory, 20, 20, &wide);
        SelectObject(memory, previous_font);
        SelectObject(memory, previous);

        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        GetDIBits(
            memory,
            bitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut info,
            DIB_RGB_COLORS,
        );
        let _ = DeleteObject(font.into());
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(memory);
        ReleaseDC(None, screen);
        for px in pixels.as_chunks_mut::<4>().0 {
            px[3] = 255;
        }
        pixels
    }

    #[test]
    #[ignore = "win32 integration: run by check.mjs --full"]
    fn recognizes_text_on_device() {
        // SAFETY: `render` only uses GDI objects it creates and releases itself.
        let pixels = unsafe { render("Please review my pull request", 900, 100) };
        let lines =
            on_mta_thread(move || recognize_pixels(&pixels, 900, 100).map_err(|e| e.to_string()))
                .unwrap()
                .unwrap();
        let text = lines.join(" ").to_lowercase();
        assert!(text.contains("review"), "recognized: {text}");
        assert!(text.contains("pull request"), "recognized: {text}");
    }

    /// Each OCR runs on a new COM thread. A call that fails early has only touched OcrEngine's
    /// cached factory; without `keep_mta_alive` the next call used it after its DLL was
    /// unloaded and crashed the app.
    #[test]
    #[ignore = "win32 integration: run by check.mjs --full"]
    fn recognizes_after_a_failed_call_on_another_thread() {
        let empty = Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        assert!(recognize(empty).is_err());
        // SAFETY: `render` only uses GDI objects it creates and releases itself.
        let pixels = unsafe { render("Again", 300, 100) };
        let lines =
            on_mta_thread(move || recognize_pixels(&pixels, 300, 100).map_err(|e| e.to_string()))
                .unwrap()
                .unwrap();
        assert_eq!(lines.join(" ").trim(), "Again");
    }

    #[test]
    #[ignore = "win32 integration: run by check.mjs --full"]
    fn an_empty_rectangle_is_an_error() {
        let empty = Rect {
            left: 10,
            top: 10,
            right: 10,
            bottom: 40,
        };
        assert!(recognize(empty).is_err());
    }
}
