// Majority of code below reproduced from https://github.com/Smithay/client-toolkit/blob/master/examples/simple_window.rs

use super::ModifiersState;
use anyhow::Result;
use log::debug;
use std::{convert::TryInto, time::Duration};

use sctk::activation::RequestData;
use sctk::reexports::calloop::EventLoop;
use sctk::reexports::calloop_wayland_source::WaylandSource;
use sctk::reexports::client::{
  globals::registry_queue_init,
  protocol::{wl_keyboard, wl_output, wl_seat, wl_shm, wl_surface},
  Connection, QueueHandle,
};
use sctk::{
  activation::{ActivationHandler, ActivationState},
  compositor::{CompositorHandler, CompositorState},
  delegate_activation, delegate_compositor, delegate_keyboard, delegate_output, delegate_registry,
  delegate_seat, delegate_shm, delegate_xdg_shell, delegate_xdg_window,
  output::{OutputHandler, OutputState},
  registry::{ProvidesRegistryState, RegistryState},
  registry_handlers,
  seat::{
    keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
    Capability, SeatHandler, SeatState,
  },
  shell::{
    xdg::{
      window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
      XdgShell,
    },
    WaylandSurface,
  },
  shm::{
    slot::{Buffer, SlotPool},
    Shm, ShmHandler,
  },
};

pub fn get_modifiers_state() -> Result<Option<super::ModifiersState>> {
  let conn = Connection::connect_to_env().unwrap();

  // Enumerate the list of globals to get the protocols the server implements.
  let (globals, event_queue) = registry_queue_init(&conn).unwrap();

  let qh = event_queue.handle();

  let mut event_loop: EventLoop<SimpleWindow> =
    EventLoop::try_new().expect("Failed to initialize the event loop!");

  let loop_handle = event_loop.handle();

  WaylandSource::new(conn.clone(), event_queue)
    .insert(loop_handle)
    .unwrap();

  // The compositor (not to be confused with the server which is commonly called the compositor) allows
  // configuring surfaces to be presented.
  let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
  // For desktop platforms, the XDG shell is the standard protocol for creating desktop windows.
  let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg shell is not available");
  // Since we are not using the GPU in this example, we use wl_shm to allow software rendering to a buffer
  // we share with the compositor process.
  let shm = Shm::bind(&globals, &qh).expect("wl shm is not available.");
  // If the compositor supports xdg-activation it probably wants us to use it to get focus
  let xdg_activation = ActivationState::bind(&globals, &qh).ok();

  // A window is created from a surface.
  let surface = compositor.create_surface(&qh);
  // And then we can create the window.
  let window = xdg_shell.create_window(surface, WindowDecorations::RequestServer, &qh);
  // Configure the window, this may include hints to the compositor about the desired minimum size of the
  // window, app id for WM identification, the window title, etc.
  window.set_title("Espanso Sync Tool");
  // GitHub does not let projects use the `org.github` domain but the `io.github` domain is fine.
  window.set_app_id("Espanso.SyncTool");
  window.set_min_size(Some((256, 256)));

  // In order for the window to be mapped, we need to perform an initial commit with no attached buffer.
  // For more info, see WaylandSurface::commit
  //
  // The compositor will respond with an initial configure that we can then use to present to the window with
  // the correct options.
  window.commit();

  // To request focus, we first need to request a token
  if let Some(activation) = xdg_activation.as_ref() {
    activation.request_token(
      &qh,
      RequestData {
        seat_and_serial: None,
        surface: Some(window.wl_surface().clone()),
        app_id: Some(String::from("Espanso.SyncTool")),
      },
    );
  }

  // We don't know how large the window will be yet, so lets assume the minimum size we suggested for the
  // initial memory allocation.
  let pool = SlotPool::new(256 * 256 * 4, &shm).expect("Failed to create pool");

  let mut simple_window = SimpleWindow {
    // Seats and outputs may be hotplugged at runtime, therefore we need to setup a registry state to
    // listen for seats and outputs.
    registry_state: RegistryState::new(&globals),
    seat_state: SeatState::new(&globals, &qh),
    output_state: OutputState::new(&globals, &qh),
    shm,
    xdg_activation,

    exit: false,
    first_configure: true,
    pool,
    width: 256,
    height: 256,
    shift: None,
    buffer: None,
    window,
    keyboard: None,
    keyboard_focus: false,
    modifiers: None,
  };

  conn.flush().unwrap();

  loop {
    if simple_window.modifiers.is_some() {
      debug!("{:?}", simple_window.modifiers);
      return Ok(simple_window.modifiers);
    }

    event_loop
      .dispatch(Duration::from_millis(16), &mut simple_window)
      .unwrap();
  }
  // We don't draw immediately, the configure will notify us when to first draw.
}

struct SimpleWindow {
  registry_state: RegistryState,
  seat_state: SeatState,
  output_state: OutputState,
  shm: Shm,
  xdg_activation: Option<ActivationState>,

  exit: bool,
  first_configure: bool,
  pool: SlotPool,
  width: u32,
  height: u32,
  shift: Option<u32>,
  buffer: Option<Buffer>,
  window: Window,
  keyboard: Option<wl_keyboard::WlKeyboard>,
  keyboard_focus: bool,
  modifiers: Option<ModifiersState>,
}

impl CompositorHandler for SimpleWindow {
  fn scale_factor_changed(
    &mut self,
    _conn: &Connection,
    _qh: &QueueHandle<Self>,
    _surface: &wl_surface::WlSurface,
    _new_factor: i32,
  ) {
    // Not needed for this example.
  }

  fn transform_changed(
    &mut self,
    _conn: &Connection,
    _qh: &QueueHandle<Self>,
    _surface: &wl_surface::WlSurface,
    _new_transform: wl_output::Transform,
  ) {
    // Not needed for this example.
  }

  fn frame(
    &mut self,
    conn: &Connection,
    qh: &QueueHandle<Self>,
    _surface: &wl_surface::WlSurface,
    _time: u32,
  ) {
    self.draw(conn, qh);
  }
}

impl OutputHandler for SimpleWindow {
  fn output_state(&mut self) -> &mut OutputState {
    &mut self.output_state
  }

  fn new_output(
    &mut self,
    _conn: &Connection,
    _qh: &QueueHandle<Self>,
    _output: wl_output::WlOutput,
  ) {
  }

  fn update_output(
    &mut self,
    _conn: &Connection,
    _qh: &QueueHandle<Self>,
    _output: wl_output::WlOutput,
  ) {
  }

  fn output_destroyed(
    &mut self,
    _conn: &Connection,
    _qh: &QueueHandle<Self>,
    _output: wl_output::WlOutput,
  ) {
  }
}

impl WindowHandler for SimpleWindow {
  fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
    self.exit = true;
  }

  fn configure(
    &mut self,
    conn: &Connection,
    qh: &QueueHandle<Self>,
    _window: &Window,
    configure: WindowConfigure,
    _serial: u32,
  ) {
    debug!("Window configured to: {:?}", configure);

    self.buffer = None;
    self.width = configure.new_size.0.map_or(256, std::num::NonZeroU32::get);
    self.height = configure.new_size.1.map_or(256, std::num::NonZeroU32::get);

    // Initiate the first draw.
    if self.first_configure {
      self.first_configure = false;
      self.draw(conn, qh);
    }
  }
}

impl ActivationHandler for SimpleWindow {
  type RequestData = RequestData;

  fn new_token(&mut self, token: String, _data: &Self::RequestData) {
    self
      .xdg_activation
      .as_ref()
      .unwrap()
      .activate::<SimpleWindow>(self.window.wl_surface(), token);
  }
}

impl SeatHandler for SimpleWindow {
  fn seat_state(&mut self) -> &mut SeatState {
    &mut self.seat_state
  }

  fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

  fn new_capability(
    &mut self,
    _conn: &Connection,
    qh: &QueueHandle<Self>,
    seat: wl_seat::WlSeat,
    capability: Capability,
  ) {
    if capability == Capability::Keyboard && self.keyboard.is_none() {
      debug!("Set keyboard capability");
      let keyboard = self
        .seat_state
        .get_keyboard(qh, &seat, None)
        .expect("Failed to create keyboard");

      self.keyboard = Some(keyboard);
    }
  }

  fn remove_capability(
    &mut self,
    _conn: &Connection,
    _: &QueueHandle<Self>,
    _: wl_seat::WlSeat,
    capability: Capability,
  ) {
    if capability == Capability::Keyboard && self.keyboard.is_some() {
      debug!("Unset keyboard capability");
      self.keyboard.take().unwrap().release();
    }
  }

  fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for SimpleWindow {
  fn enter(
    &mut self,
    _: &Connection,
    _: &QueueHandle<Self>,
    _: &wl_keyboard::WlKeyboard,
    surface: &wl_surface::WlSurface,
    _: u32,
    _: &[u32],
    keysyms: &[Keysym],
  ) {
    if self.window.wl_surface() == surface {
      debug!("Keyboard focus on window with pressed syms: {keysyms:?}");
      self.keyboard_focus = true;
    }
  }

  fn leave(
    &mut self,
    _: &Connection,
    _: &QueueHandle<Self>,
    _: &wl_keyboard::WlKeyboard,
    surface: &wl_surface::WlSurface,
    _: u32,
  ) {
    if self.window.wl_surface() == surface {
      debug!("Release keyboard focus on window");
      self.keyboard_focus = false;
    }
  }

  fn press_key(
    &mut self,
    _conn: &Connection,
    _qh: &QueueHandle<Self>,
    _: &wl_keyboard::WlKeyboard,
    _: u32,
    event: KeyEvent,
  ) {
    debug!("Key press: {event:?}");
  }

  fn release_key(
    &mut self,
    _: &Connection,
    _: &QueueHandle<Self>,
    _: &wl_keyboard::WlKeyboard,
    _: u32,
    event: KeyEvent,
  ) {
    debug!("Key release: {event:?}");
  }

  fn update_modifiers(
    &mut self,
    _: &Connection,
    _: &QueueHandle<Self>,
    _: &wl_keyboard::WlKeyboard,
    _serial: u32,
    modifiers: Modifiers,
  ) {
    debug!("Update modifiers: {modifiers:?}");
    self.modifiers = Some(ModifiersState {
      ctrl: modifiers.ctrl,
      alt: modifiers.alt,
      shift: modifiers.shift,
      caps_lock: modifiers.caps_lock,
      meta: modifiers.logo,
      num_lock: modifiers.num_lock,
    });
  }
}

impl ShmHandler for SimpleWindow {
  fn shm_state(&mut self) -> &mut Shm {
    &mut self.shm
  }
}

#[allow(clippy::many_single_char_names)]
#[allow(clippy::single_match_else)]
impl SimpleWindow {
  pub fn draw(&mut self, _conn: &Connection, qh: &QueueHandle<Self>) {
    let width = self.width;
    let height = self.height;
    let stride = self.width as i32 * 4;

    let buffer = self.buffer.get_or_insert_with(|| {
      self
        .pool
        .create_buffer(
          width as i32,
          height as i32,
          stride,
          wl_shm::Format::Argb8888,
        )
        .expect("create buffer")
        .0
    });

    let canvas = match self.pool.canvas(buffer) {
      Some(canvas) => canvas,
      None => {
        // This should be rare, but if the compositor has not released the previous
        // buffer, we need double-buffering.
        let (second_buffer, canvas) = self
          .pool
          .create_buffer(
            self.width as i32,
            self.height as i32,
            stride,
            wl_shm::Format::Argb8888,
          )
          .expect("create buffer");
        *buffer = second_buffer;
        canvas
      }
    };

    // Draw to the window:
    {
      let shift = self.shift.unwrap_or(0);
      canvas
        .chunks_exact_mut(4)
        .enumerate()
        .for_each(|(index, chunk)| {
          let x = ((index + shift as usize) % width as usize) as u32;
          let y = (index / width as usize) as u32;

          let a = 0xFF;
          let r = u32::min(((width - x) * 0xFF) / width, ((height - y) * 0xFF) / height);
          let g = u32::min((x * 0xFF) / width, ((height - y) * 0xFF) / height);
          let b = u32::min(((width - x) * 0xFF) / width, (y * 0xFF) / height);
          let color = (a << 24) + (r << 16) + (g << 8) + b;

          let array: &mut [u8; 4] = chunk.try_into().unwrap();
          *array = color.to_le_bytes();
        });

      if let Some(shift) = &mut self.shift {
        *shift = (*shift + 1) % width;
      }
    }

    // Damage the entire window
    self
      .window
      .wl_surface()
      .damage_buffer(0, 0, self.width as i32, self.height as i32);

    // Request our next frame
    self
      .window
      .wl_surface()
      .frame(qh, self.window.wl_surface().clone());

    // Attach and commit to present.
    buffer
      .attach_to(self.window.wl_surface())
      .expect("buffer attach");
    self.window.commit();
  }
}

delegate_compositor!(SimpleWindow);
delegate_output!(SimpleWindow);
delegate_shm!(SimpleWindow);

delegate_seat!(SimpleWindow);
delegate_keyboard!(SimpleWindow);

delegate_xdg_shell!(SimpleWindow);
delegate_xdg_window!(SimpleWindow);
delegate_activation!(SimpleWindow);

delegate_registry!(SimpleWindow);

impl ProvidesRegistryState for SimpleWindow {
  fn registry(&mut self) -> &mut RegistryState {
    &mut self.registry_state
  }
  registry_handlers![OutputState, SeatState,];
}
