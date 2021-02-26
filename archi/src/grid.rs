use crate::context::Context;
use luminance_front::context::GraphicsContext;
use luminance_front::tess::{Mode, Tess};

pub fn square_patch_grid(
    context: &mut Context,
    side_length: u32,
) -> anyhow::Result<Tess<(), u32>> {
    let indices = {
        let capacity = {
            let side_length = side_length as usize;
            side_length * side_length * 4
        };

        let mut indices = Vec::with_capacity(capacity);

        for x in 0..side_length {
            for y in 0..side_length {
                let line_count = side_length + 1;
                indices.push(x * line_count + y);
                indices.push(x * line_count + y + 1);
                indices.push(x * line_count + y + line_count + 1);
                indices.push(x * line_count + y + line_count);
            }
        }

        assert_eq!(indices.len(), capacity);
        indices
    };

    let tess = context
        .new_tess()
        .set_mode(Mode::Patch(4))
        .set_vertex_nb(indices.len())
        .set_indices(indices)
        .build()?;

    Ok(tess)
}
