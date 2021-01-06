use alloc::string::{String, ToString};
use conquer_once::spin::OnceCell;
use fontdue::{Font, FontSettings};
use spin::Mutex;
use uefi::{
    prelude::*,
    proto::console::gop::{GraphicsOutput, PixelFormat},
};

pub enum FontSize {
    Regular,
    Large,
}

impl FontSize {
    pub fn size(&self) -> f32 {
        match self {
            FontSize::Regular => 13.0,
            FontSize::Large => 18.0,
        }
    }

    pub fn line_height(&self) -> f32 {
        self.size() + 2.0
    }
}

#[derive(Clone, Copy)]
pub enum FontStyle {
    Regular = 0,
    // Italic = 1,
    Bold = 2,
}

pub struct TextPart(pub String, pub FontSize, pub FontStyle);

pub struct Display {
    fonts: [Font; 3],
    resolution: (usize, usize),
    frame_buffer: usize,
    pixel_format: PixelFormat,
    position: (f32, f32),
}

impl Display {
    pub fn new(boot_service: &BootServices) -> Display {
        let gop = boot_service
            .locate_protocol::<GraphicsOutput>()
            .expect_success("Could not find graphics protocol");
        let gop = unsafe { &mut *gop.get() };

        let mode_info = gop.current_mode_info();
        let resolution = mode_info.resolution();
        let frame_buffer = gop.frame_buffer().as_mut_ptr() as usize;

        let fonts = [
            Font::from_bytes(
                include_bytes!("../../fonts/FiraSans-Light.ttf") as &[u8],
                FontSettings::default(),
            )
            .unwrap(),
            Font::from_bytes(
                include_bytes!("../../fonts/FiraSans-LightItalic.ttf") as &[u8],
                FontSettings::default(),
            )
            .unwrap(),
            Font::from_bytes(
                include_bytes!("../../fonts/FiraSans-Bold.ttf") as &[u8],
                FontSettings::default(),
            )
            .unwrap(),
        ];

        Display {
            fonts,
            resolution,
            frame_buffer,
            pixel_format: mode_info.pixel_format(),
            position: (0., 0.),
        }
    }

    pub fn clear_screen(&mut self) {
        let frame_buffer = self.frame_buffer as *mut u8;
        for x in 0..self.resolution.0 {
            for y in 0..self.resolution.1 {
                let index = (x + (y * self.resolution.0)) as usize;
                unsafe {
                    *frame_buffer.add(index + 0) = 0;
                    *frame_buffer.add(index + 1) = 0;
                    *frame_buffer.add(index + 2) = 0;
                    *frame_buffer.add(index + 3) = 0;
                }
            }
        }
    }

    pub fn write_text(&mut self, text: TextPart) {
        for c in text.0.chars() {
            match c {
                '\n' => self.write_newline(&text.1),
                c => self.write_char(c, &text.1, &text.2),
            }
        }
    }

    pub fn write_newline(&mut self, size: &FontSize) {
        self.position.0 = 0.;
        self.position.1 += size.line_height();
    }

    pub fn write_char(&mut self, c: char, size: &FontSize, style: &FontStyle) {
        let frame_buffer = self.frame_buffer as *mut u8;
        let font = &self.fonts[*style as usize];
        let (metrics, bitmap) = font.rasterize(c, size.size());

        let offset_y = size.size() - metrics.height as f32 - metrics.ymin as f32;

        for pixel_y in 0..metrics.height {
            for pixel_x in 0..metrics.width {
                let red = bitmap[0 + pixel_x + pixel_y * metrics.width];
                let green = bitmap[0 + pixel_x + pixel_y * metrics.width];
                let blue = bitmap[0 + pixel_x + pixel_y * metrics.width];

                let position = (
                    self.position.0 + (pixel_x as f32),
                    self.position.1 + (pixel_y as f32) + offset_y,
                );

                let index = (position.0 + (position.1 * self.resolution.0 as f32)) as usize;

                unsafe {
                    match self.pixel_format {
                        PixelFormat::BGR => {
                            *frame_buffer.add((4 * index) + 0) = blue;
                            *frame_buffer.add((4 * index) + 1) = green;
                            *frame_buffer.add((4 * index) + 2) = red;
                            *frame_buffer.add((4 * index) + 3) = 0;
                        }

                        PixelFormat::RGB => {
                            *frame_buffer.add((4 * index) + 0) = red;
                            *frame_buffer.add((4 * index) + 1) = green;
                            *frame_buffer.add((4 * index) + 2) = blue;
                            *frame_buffer.add((4 * index) + 3) = 0;
                        }

                        _ => {}
                    }
                }
            }
        }

        self.position.0 += metrics.advance_width;
        self.position.1 += metrics.advance_height;
    }
}

impl core::fmt::Write for Display {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_text(TextPart(
            s.to_string(),
            FontSize::Regular,
            FontStyle::Regular,
        ));
        Ok(())
    }
}

pub static DISPLAY: OnceCell<Mutex<Display>> = OnceCell::uninit();

pub fn init(system_table: &SystemTable<Boot>) {
    DISPLAY.init_once(|| Mutex::new(Display::new(system_table.boot_services())));
    clear_screen();
}

pub fn clear_screen() {
    DISPLAY.get().unwrap().lock().clear_screen();
}

pub fn write_text(text: TextPart) {
    DISPLAY.get().unwrap().lock().write_text(text);
}
