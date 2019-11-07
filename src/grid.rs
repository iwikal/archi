use luminance::{
    context::GraphicsContext,
    tess::{Mode, Tess, TessBuilder},
};

pub fn grid(context: &mut impl GraphicsContext, side_length: usize) -> Tess {
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
        .set_mode(Mode::TriangleStrip)
        .set_primitive_restart_index(Some(restart))
        .set_vertex_nb(indices.len())
        .set_indices(indices)
        .build()
        .unwrap()
}
