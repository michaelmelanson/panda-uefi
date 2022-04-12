#![no_std]

pub enum PixelFormat {
    BGR,
    RGB,
}

pub struct FrameBuffer {
    pub base_addr: usize,
    pub resolution: (usize, usize),
    pub stride: usize,
    pub pixel_format: PixelFormat,
}

impl FrameBuffer {
    fn ptr(&self) -> *mut u8 {
        self.base_addr as *mut u8
    }
    pub fn read_pixel(&self, position: (usize, usize)) -> (u8, u8, u8) {
        if position.0 >= self.resolution.0 || position.1 >= self.resolution.1 {
            return (0, 0, 0);
        }

        let index = position.0 + (position.1 * self.resolution.0);

        unsafe {
            match self.pixel_format {
                PixelFormat::BGR => {
                    let blue = *self.ptr().add((4 * index) + 0);
                    let green = *self.ptr().add((4 * index) + 1);
                    let red = *self.ptr().add((4 * index) + 2);

                    (red, green, blue)
                }

                PixelFormat::RGB => {
                    let red = *self.ptr().add((4 * index) + 0);
                    let green = *self.ptr().add((4 * index) + 1);
                    let blue = *self.ptr().add((4 * index) + 2);

                    (red, green, blue)
                }

                _ => (0, 0, 0),
            }
        }
    }

    pub fn draw_pixel(&mut self, position: (usize, usize), colour: (u8, u8, u8)) {
        if position.0 > self.resolution.0 {
            return;
        }

        if position.1 > self.resolution.1 {
            return;
        }

        let (red, green, blue) = colour;
        let index = position.0 + (position.1 * self.resolution.0);

        match self.pixel_format {
            PixelFormat::BGR => unsafe {
                *self.ptr().add((4 * index) + 0) = blue;
                *self.ptr().add((4 * index) + 1) = green;
                *self.ptr().add((4 * index) + 2) = red;
                *self.ptr().add((4 * index) + 3) = 0;
            },

            PixelFormat::RGB => unsafe {
                *self.ptr().add((4 * index) + 0) = red;
                *self.ptr().add((4 * index) + 1) = green;
                *self.ptr().add((4 * index) + 2) = blue;
                *self.ptr().add((4 * index) + 3) = 0;
            },

            _ => {}
        }
    }
}

pub struct LoaderCarePackage {
    pub frame_buffer: FrameBuffer,
}
