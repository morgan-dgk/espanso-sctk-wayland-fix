use {
    std::usize,
    log::debug,
    wayland_client::{
        protocol::{wl_keyboard, wl_registry, wl_seat},
        Connection, Dispatch, QueueHandle, WEnum,
    },
    xkbcommon::xkb,
};


fn read_xkb_keymap(fd: OwnedFd) -> () {
    let mut f = unsafe { File::from_raw_fd(fd.as_raw_fd()) };
    let mut map = String::new();
    f.rewind().unwrap();
    f.read_to_string(&mut map).unwrap();
    println!("{:?}", map);
}

#[derive(Debug)]
struct AppData;

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        _state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        // When receiving events from the wl_registry, we are only interested in the
        // `global` event, which signals a new available global.
        // When receiving this event, we just print its characteristics in this example.
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match &interface[..] {
                "wl_seat" => {
                    registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
                }

                _ => {}
            }
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for AppData {
    fn event(
        _state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        if let wl_keyboard::Event::Keymap { format: WEnum::Value(format), fd, size } = event {
            match format {
                wl_keyboard::KeymapFormat::XkbV1 => read_xkb_keymap(fd),
                _ => ()
            }
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for AppData {
    fn event(
        _state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        {
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                debug!("Setting keyboard capability");
                seat.get_keyboard(qh, ());
            }
        }
    }
}

fn main() {
    let conn = Connection::connect_to_env().unwrap();
    println!("Successfully connected!");

    // Retrieve the WlDisplay Wayland object from the connection. This object is
    // the starting point of any Wayland program, from which all other objects will
    // be created.
    let display = conn.display();

    // Create an event queue for our event processing
    let mut event_queue = conn.new_event_queue();
    // And get its handle to associate new objects to it
    let qh = event_queue.handle();

    let mut app_state = AppData;

    println!("{:?}", app_state);

    // To actually receive the events, we invoke the `roundtrip` method. This method
    // is special and you will generally only invoke it during the setup of your program:
    // it will block until the server has received and processed all the messages you've
    // sent up to now.
    
    event_queue.roundtrip(&mut app_state).unwrap();

    loop {
        event_queue.blocking_dispatch(&mut app_state).unwrap();
    }
}
