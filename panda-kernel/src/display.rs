use conquer_once::spin::OnceCell;
use fontdue::{Font, FontSettings};
use panda_loader_lib::FrameBuffer;
use spin::Mutex;

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

    pub fn space_width(&self) -> f32 {
        match self {
            FontSize::Regular => 3.0,
            FontSize::Large => 4.0,
        }
    }

    pub fn tab_width(&self) -> f32 {
        4.0 * self.space_width()
    }
}

#[derive(Clone, Copy)]
pub enum FontStyle {
    Regular = 0,
    // Italic = 1,
    Bold = 2,
}

pub struct TextPart<'a>(pub &'a str, pub FontSize, pub FontStyle);

pub struct Display {
    frame_buffer: FrameBuffer,
    fonts: [Font; 3],
    position: (usize, usize),
}

impl Display {
    pub fn new(frame_buffer: FrameBuffer) -> Display {
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
            frame_buffer,
            fonts,
            position: (0, 0),
        }
    }

    pub fn clear_screen(&mut self) {
        for x in 0..self.frame_buffer.resolution.0 {
            for y in 0..self.frame_buffer.resolution.1 {
                self.frame_buffer.draw_pixel((x, y), (0, 0, 0));
            }
        }
    }

    pub fn write_text(&mut self, text: TextPart) {
        for c in text.0.chars() {
            self.write_char(c, &text.1, &text.2);
        }
    }

    pub fn scroll_up(&mut self, offset: usize) {
        for pixel_y in 0..self.frame_buffer.resolution.1 {
            for pixel_x in 0..self.frame_buffer.resolution.0 {
                let source_position = (pixel_x, pixel_y + offset);
                let destination_position = (pixel_x, pixel_y);

                let colour = self.frame_buffer.read_pixel(source_position);

                self.frame_buffer.draw_pixel(destination_position, colour);
            }
        }

        self.position.1 -= offset;
    }

    pub fn write_newline(&mut self, size: &FontSize) {
        self.position.0 = 0;
        self.position.1 += size.line_height() as usize;
    }

    pub fn write_char(&mut self, c: char, size: &FontSize, style: &FontStyle) {
        match c {
            ' ' => self.position.0 += size.space_width() as usize,
            '\t' => self.position.0 += size.tab_width() as usize,
            '\n' => self.write_newline(size),
            c => {
                let font = &self.fonts[*style as usize];
                let (metrics, bitmap) = font.rasterize(c, size.size());

                if self.position.0 + metrics.width > self.frame_buffer.resolution.0 {
                    self.write_newline(size);
                }

                let offset_y =
                    size.size() as usize - metrics.height as usize - metrics.ymin as usize;

                if self.position.1 + metrics.height + offset_y > self.frame_buffer.resolution.1 {
                    let diff = (self.position.1 + metrics.height + offset_y)
                        - self.frame_buffer.resolution.1;
                    self.scroll_up(diff);
                }

                for pixel_y in 0..metrics.height {
                    for pixel_x in 0..metrics.width {
                        let coverage = bitmap[pixel_x + pixel_y * metrics.width];
                        let colour = (coverage, coverage, coverage);

                        let position = (
                            self.position.0 + pixel_x,
                            self.position.1 + pixel_y + offset_y,
                        );

                        self.frame_buffer.draw_pixel(position, colour);
                    }
                }

                self.position.0 += metrics.width as usize;
            }
        }
    }
}

impl core::fmt::Write for Display {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_text(TextPart(s, FontSize::Regular, FontStyle::Regular));
        Ok(())
    }
}

pub static DISPLAY: OnceCell<Mutex<Display>> = OnceCell::uninit();

pub fn init(frame_buffer: FrameBuffer) {
    let display = Display::new(frame_buffer);
    DISPLAY.init_once(|| Mutex::new(display));
    clear_screen();
}

pub fn clear_screen() {
    DISPLAY.get().unwrap().lock().clear_screen();
}

pub fn write_text(text: TextPart) {
    DISPLAY.get().unwrap().lock().write_text(text);
}
