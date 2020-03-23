use crate::{
    colors::{Color16Bit, DEFAULT_PALETTE},
    registers::PlaneMask,
    vga::{Vga, VideoMode, VGA},
};
use core::convert::TryInto;
use spinning_top::SpinlockGuard;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const ALL_PLANES_SCREEN_SIZE: usize = (WIDTH * HEIGHT) / 4;

/// A basic interface for interacting with vga graphics mode 640x480x16
///
/// # Examples
///
/// Basic usage:
///
/// ```no_run
/// use vga::writers::Graphics640x480x16;
///
/// let graphics_mode = Graphics640x480x16::new();
///
/// graphics_mode.set_mode();
/// graphics_mode.clear_screen();
/// ```
#[derive(Default)]
pub struct Graphics640x480x16;

impl Graphics640x480x16 {
    /// Creates a new `Graphics640x480x16`.
    pub fn new() -> Graphics640x480x16 {
        Graphics640x480x16 {}
    }

    /// Clears the screen by setting all pixels to `Color16Bit::Black`.
    pub fn clear_screen(&self) {
        let (mut vga, frame_buffer) = self.get_frame_buffer();
        vga.sequencer_registers
            .set_plane_mask(PlaneMask::ALL_PLANES);
        vga.graphics_controller_registers
            .write_enable_set_reset(PlaneMask::NONE);
        for offset in 0..ALL_PLANES_SCREEN_SIZE {
            unsafe {
                frame_buffer
                    .add(offset)
                    .write_volatile(Color16Bit::Black as u8);
            }
        }
    }

    /// Sets the given pixel at `(x, y)` to the given `color`.
    pub fn set_pixel(&self, x: usize, y: usize, color: Color16Bit) {
        let (mut vga, frame_buffer) = self.get_frame_buffer();
        let offset = x / 8 + (WIDTH / 8) * y;

        // Store the current value for masking.
        let x = x & 7;
        let mask = 0x80 >> (x & 7);
        let mut plane_mask = 0x01;

        for plane in 0u8..4u8 {
            vga.graphics_controller_registers
                .write_read_plane(plane.try_into().unwrap());
            vga.sequencer_registers
                .set_plane_mask(plane.try_into().unwrap());
            let current_value = unsafe { frame_buffer.add(offset).read_volatile() };
            let new_value = if plane_mask & color as u8 != 0 {
                current_value | mask
            } else {
                current_value & !mask
            };
            unsafe {
                frame_buffer.add(offset).write_volatile(new_value);
            }
            plane_mask <<= 1;
        }
    }

    /// Sets the graphics device to `VideoMode::Mode640x480x16`.
    pub fn set_mode(&self) {
        let mut vga = VGA.lock();
        vga.set_video_mode(VideoMode::Mode640x480x16);

        // Some bios mess up the palette when switching modes,
        // so explicitly set it.
        vga.color_palette_registers.load_palette(&DEFAULT_PALETTE);
    }

    /// Returns the start of the `FrameBuffer` as `*mut u8` as
    /// well as a lock to the vga driver. This ensures the vga
    /// driver stays locked while the frame buffer is in use.
    fn get_frame_buffer(&self) -> (SpinlockGuard<Vga>, *mut u8) {
        let mut vga = VGA.lock();
        let frame_buffer = vga.get_frame_buffer();
        (vga, u32::from(frame_buffer) as *mut u8)
    }
}
