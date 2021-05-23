use crate::context::Context;
use luminance_front::{
    context::GraphicsContext,
    pixel::RGBA32F,
    texture::{Dim2, Texture},
};

pub struct BlueNoise {
    pub freq_texture: Texture<Dim2, RGBA32F>,
    pub noise_texture: Texture<Dim2, RGBA32F>,
}

impl BlueNoise {
    pub fn new(context: &mut Context) -> anyhow::Result<Self> {
        use luminance::texture::{GenMipmaps, MagFilter, MinFilter, Sampler};
        let sampler = Sampler {
            mag_filter: MagFilter::Nearest,
            min_filter: MinFilter::Nearest,
            ..Default::default()
        };

        let size = 128;
        let mut freq_texture = {
            let mut texture = Texture::new(context, [size, size], 0, sampler)?;
            let size = size as usize;
            let mut pixels = Vec::with_capacity(size * size);
            for x in 0..size {
                for y in 0..size {
                    pixels.push(match (x, y) {
                        (0, 0) => (1., 1., 0., 0.),
                        (x, y) => {
                            let scale = 1. / 256.;
                            let x = x as f32 * scale;
                            let y = y as f32 * scale;
                            let mag_sq = (x * x + y * y).sqrt();

                            let val =
                                || mag_sq * (rand::random::<f32>() * 2. - 1.);
                            (val(), val(), val(), val())
                        }
                    });
                }
            }

            texture.upload(GenMipmaps::No, &pixels)?;
            texture
        };

        let mut fft = crate::fft::Fft::with_sampler(context, size, sampler)?;
        fft.render(&mut context.new_pipeline_gate(), &mut freq_texture)?;
        let noise_texture = fft.into_target_texture();

        Ok(Self {
            freq_texture,
            noise_texture,
        })
    }
}
