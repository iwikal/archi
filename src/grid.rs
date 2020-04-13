use crate::context::Context;
use luminance::tess::{Mode, Tess, TessBuilder};
use luminance_gl::GL33;

pub fn strip_grid(context: &mut Context, side_length: usize) -> Tess<GL33> {
    let line_count = side_length + 1;

    let restart = u32::max_value();
    let indices = {
        let mut indices =
            Vec::with_capacity(side_length * (line_count * 2 + 1) - 1);
        let side_length = side_length as u32;
        let line_count = line_count as u32;
        for x in 0..side_length {
            if x != 0 {
                indices.push(restart);
            }
            for y in 0..line_count {
                indices.push(x * line_count + y);
                indices.push(x * line_count + y + line_count);
            }
        }
        assert_eq!(indices.len(), indices.capacity());
        indices
    };

    TessBuilder::new(context)
        .and_then(|b| b.set_mode(Mode::TriangleStrip))
        .and_then(|b| b.set_primitive_restart_index(Some(restart)))
        .and_then(|b| b.set_vertex_nb(indices.len()))
        .and_then(|b| b.set_indices(indices))
        .and_then(|b| b.build())
        .unwrap()
}

pub fn square_patch_grid(
    context: &mut Context,
    side_length: u32,
) -> Tess<GL33> {
    let indices = {
        let mut indices = Vec::with_capacity({
            let side_length = side_length as usize;
            side_length * side_length * 4
        });

        for x in 0..side_length {
            for y in 0..side_length {
                let line_count = side_length + 1;
                indices.push(x * line_count + y);
                indices.push(x * line_count + y + 1);
                indices.push(x * line_count + y + line_count + 1);
                indices.push(x * line_count + y + line_count);
            }
        }
        assert_eq!(indices.len(), indices.capacity());
        indices
    };

    TessBuilder::new(context)
        .and_then(|b| b.set_mode(Mode::Patch(4)))
        .and_then(|b| b.set_vertex_nb(indices.len()))
        .and_then(|b| b.set_indices(indices))
        .and_then(|b| b.build())
        .unwrap()
}
