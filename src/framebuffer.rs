pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    background_color: u32,
    current_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; width * height],
            background_color: 0x000000,
            current_color: 0xFFFFFF,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.fill(self.background_color);
    }

    pub fn point(&mut self, x: i32, y: i32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.buffer[y as usize * self.width + x as usize] = self.current_color;
        }
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color & 0xFFFFFF;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color & 0xFFFFFF;
    }

    pub fn to_rgb_bytes(&self) -> Vec<u8> {
        self.buffer.iter().flat_map(|&pixel| {
            [(pixel >> 16) as u8, (pixel >> 8) as u8, pixel as u8]
        }).collect()
    }

    // A circular stamp with an exact bounding diameter, including even sizes.
    pub fn stamp(&mut self, x: i32, y: i32, diameter: i32) {
        let diameter = diameter.max(1);
        let start = -(diameter / 2);
        let end = start + diameter;
        let offset = if diameter % 2 == 0 { 1 } else { 0 };
        for dy in start..end {
            for dx in start..end {
                let cx = 2 * dx + offset;
                let cy = 2 * dy + offset;
                if cx * cx + cy * cy <= diameter * diameter {
                    self.point(x + dx, y + dy);
                }
            }
        }
    }

    // Bresenham interpolation prevents gaps when the pointer moves quickly.
    pub fn thick_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, diameter: i32) {
        if diameter <= 1 {
            crate::line::draw_line(x0, y0, x1, y1, self);
            return;
        }
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.stamp(x, y, diameter);
            if (x, y) == (x1, y1) { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x += sx; }
            if e2 <= dx { err += dx; y += sy; }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_sizes_have_exact_width_and_clip_at_edges() {
        for diameter in [1, 3, 6, 12, 20] {
            let mut fb = Framebuffer::new(50, 50);
            fb.stamp(25, 25, diameter);
            let xs: Vec<usize> = fb.buffer.iter().enumerate()
                .filter(|(_, c)| **c != 0).map(|(i, _)| i % 50).collect();
            assert_eq!(xs.iter().max().unwrap() - xs.iter().min().unwrap() + 1, diameter as usize);
            fb.stamp(0, 0, diameter);
            fb.stamp(49, 49, diameter);
            assert_eq!(fb.buffer.len(), 2500);
        }
    }

    #[test]
    fn fast_diagonal_stroke_is_continuous_and_eraser_restores_background() {
        let mut fb = Framebuffer::new(100, 100);
        fb.set_background_color(0xFFFFFF);
        fb.clear();
        fb.set_current_color(0x1299EF);
        fb.thick_line(0, 0, 99, 99, 6);
        for i in 0..100 { assert_eq!(fb.buffer[i * 100 + i], 0x1299EF); }
        fb.set_current_color(0xFFFFFF);
        fb.thick_line(0, 0, 99, 99, 6);
        assert!(fb.buffer.iter().all(|&p| p == 0xFFFFFF));
    }
}
