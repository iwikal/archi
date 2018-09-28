extern crate sdl2;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_system = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let _window = video_system.window("archi", 800, 600)
        .build()
        .unwrap();

    for e in event_pump.wait_iter() {
        use sdl2::event::Event;
        match e {
            Event::Quit {..} => {
                println!("Quit");
                break;
            },
            _ => {}
        }
    }
}
