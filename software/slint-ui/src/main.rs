use slint::{SharedPixelBuffer, Rgba8Pixel, Image};
use tiny_skia::*;
use std::println;
use anyhow::Result;

mod pwm;
use crate::pwm::{Pwm, XyColor};

slint::include_modules!();

fn main() -> Result<(), anyhow::Error> {
    let ui = AppWindow::new()?;

    let background_image: Pixmap = Pixmap::load_png(&std::path::Path::new("assets/img/CIE1931xy-1927px.png"))?;

    let ui_handle = ui.as_weak();
    ui.on_request_increase_value(move || {
        let ui = ui_handle.unwrap();
        ui.set_counter(ui.get_counter() + 1);
    });

    ui.on_render_image(move |width: f32, height: f32| -> Image {
        return render_image(width, height, &background_image).unwrap_or(Image::default());
    });
   

    Ok(ui.run()?)
}

/**
 See https://docs.rs/slint/latest/slint/struct.Image.html.
This is an only slightly modified version, so that the circle is always
in the windows center, with arbitrary window sizes.
*/
fn render_image(outer_width: f32, outer_height: f32, background_image: &Pixmap) -> Option<Image> {
    let width = u32::min(outer_width as u32, outer_height as u32);
    let height = width;
    let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);

    let outer_transform = Transform::from_scale(
        width as f32 / background_image.width() as f32,
        height as f32 / background_image.height() as f32
    );
    println!("render image {} * {} with transform {:?}", width, height, outer_transform);

    let mut pixmap: PixmapMut = PixmapMut::from_bytes(
        pixel_buffer.make_mut_bytes(), width, height,
    ).expect("Should create pixel buffer");
  
    pixmap.fill(tiny_skia::Color::WHITE);

    let mut paint = tiny_skia::Paint::default();
    pixmap.draw_pixmap(0, 0, background_image.as_ref(), &Default::default(), outer_transform, None);

    let inner_transform = outer_transform.pre_translate(294.0, 1794.0+58.0).pre_scale(1.0, -1.0).pre_scale(1596.0 / 0.8, 1794.0 / 0.9);

    paint.set_color(tiny_skia::Color::BLACK);
    paint.anti_alias = true;

    let path = {
        let mut pb = PathBuilder::new();
        for i in 17..150 {
            let temp = i as f32 * 100.0;
            let color: XyColor = Pwm::temperature_to_xy(temp).expect("Should give a color");
            if pb.is_empty() {
                pb.move_to(color.x, color.y);
            } else {
                pb.line_to(color.x, color.y);
            }
        }
        pb.finish().unwrap()
    };

    let mut stroke = Stroke::default();
    stroke.width = 0.006;
    stroke.line_cap = LineCap::Round;

    pixmap.stroke_path(&path, &paint, &stroke, inner_transform, None);    
    
    Option::Some(Image::from_rgba8_premultiplied(pixel_buffer))
}
