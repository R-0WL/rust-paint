use crate::framebuffer::Framebuffer;

pub fn draw_line(x0: i32, y0: i32, x1: i32, y1: i32, fb: &mut Framebuffer) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        fb.point(x, y);

        if x == x1 && y == y1 {
            break;
        }

        let e2 = err * 2;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::draw_line;
    use crate::framebuffer::Framebuffer;

    #[test]
    fn draws_the_endpoints_of_a_simple_segment() {
        let mut fb = Framebuffer::new(100, 100);
        draw_line(0, 0, 4, 2, &mut fb);

        assert_eq!(fb.buffer[0], 0xFFFFFF);
        assert_eq!(fb.buffer[2 * 100 + 4], 0xFFFFFF);
    }
}
