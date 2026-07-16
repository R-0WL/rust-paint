mod framebuffer;
mod line;
mod bmp;

use crate::bmp::WriteBmp;
use crate::framebuffer::Framebuffer;
use crate::line::draw_line;

fn main() {
    let mut framebuffer = Framebuffer::new(450, 450);

    framebuffer.set_background_color(0x000000);
    framebuffer.clear();
    framebuffer.set_current_color(0xFFFFFF);
    // R
    draw_line(140, 25, 140, 45, &mut framebuffer);
    draw_line(140, 25, 150, 25, &mut framebuffer);
    draw_line(150, 25, 150, 34, &mut framebuffer);
    draw_line(140, 34, 150, 34, &mut framebuffer);
    draw_line(140, 34, 151, 45, &mut framebuffer);

    // -
    draw_line(155, 35, 165, 35, &mut framebuffer);

    // 0
    draw_line(170, 25, 180, 25, &mut framebuffer);
    draw_line(170, 25, 170, 45, &mut framebuffer);
    draw_line(180, 25, 180, 45, &mut framebuffer);
    draw_line(170, 45, 180, 45, &mut framebuffer);

    // W
    draw_line(185, 25, 188, 45, &mut framebuffer);
    draw_line(188, 45, 193, 35, &mut framebuffer);
    draw_line(193, 35, 198, 45, &mut framebuffer);
    draw_line(198, 45, 201, 25, &mut framebuffer);

    // L
    draw_line(207, 25, 207, 45, &mut framebuffer);
    draw_line(207, 45, 217, 45, &mut framebuffer);

    // Cabeza y cejas
    draw_line(100, 120, 225, 210, &mut framebuffer);
    draw_line(225, 210, 350, 120, &mut framebuffer);
    draw_line(100, 120, 130, 190, &mut framebuffer);
    draw_line(350, 120, 320, 190, &mut framebuffer);

    // Ojos
    draw_line(130, 190, 180, 170, &mut framebuffer);
    draw_line(180, 170, 215, 210, &mut framebuffer);
    draw_line(235, 210, 270, 170, &mut framebuffer);
    draw_line(270, 170, 320, 190, &mut framebuffer);

    // Pupila izquierda, usando líneas
    draw_line(165, 235, 180, 235, &mut framebuffer);
    draw_line(180, 235, 185, 245, &mut framebuffer);
    draw_line(185, 245, 175, 252, &mut framebuffer);
    draw_line(175, 252, 165, 245, &mut framebuffer);
    draw_line(165, 245, 165, 235, &mut framebuffer);
    
    // Pupila derecha, usando líneas
    draw_line(285, 235, 300, 235, &mut framebuffer);
    draw_line(300, 235, 300, 245, &mut framebuffer);
    draw_line(300, 245, 290, 252, &mut framebuffer);
    draw_line(290, 252, 280, 245, &mut framebuffer);
    draw_line(280, 245, 285, 235, &mut framebuffer);

    // Pico
    draw_line(210, 270, 225, 320, &mut framebuffer);
    draw_line(225, 320, 240, 270, &mut framebuffer);

    // Cuerpo
    draw_line(130, 190, 110, 280, &mut framebuffer);
    draw_line(110, 280, 160, 340, &mut framebuffer);
    draw_line(160, 340, 225, 370, &mut framebuffer);
    draw_line(225, 370, 290, 340, &mut framebuffer);
    draw_line(290, 340, 340, 280, &mut framebuffer);
    draw_line(340, 280, 320, 190, &mut framebuffer);

    // Plumas
    draw_line(140, 260, 190, 320, &mut framebuffer);
    draw_line(190, 320, 225, 350, &mut framebuffer);
    draw_line(310, 260, 260, 320, &mut framebuffer);
    draw_line(260, 320, 225, 350, &mut framebuffer);

    let _ = framebuffer.render_buffer("owl.bmp");
    let _ = framebuffer.render_buffer("out.png");

    println!("Búho dibujado en owl.bmp");
}