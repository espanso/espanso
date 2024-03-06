use anyhow::Result;

use super::ModifiersState;
use log::debug;
use std::time::Duration;

use sctk::{
  delegate_keyboard, delegate_registry, delegate_seat,
  reexports::calloop::{EventLoop, LoopHandle},
  reexports::calloop_wayland_source::WaylandSource,
  reexports::client::protocol::{wl_keyboard, wl_seat, wl_surface},
  registry::{ProvidesRegistryState, RegistryHandler, RegistryState},
  registry_handlers,
  seat::{
    keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
    Capability, SeatHandler, SeatState,
  },
};

use sctk::reexports::client::{globals::registry_queue_init, Connection, QueueHandle};

struct SyncToolState {
  registry_state: RegistryState,
  seat_state: SeatState,
  keyboard: Option<wl_keyboard::WlKeyboard>,
  loop_handle: LoopHandle<'static, SyncToolState>,
  modifiers: Option<super::ModifiersState>,
}

pub fn get_modifiers_state() -> Result<Option<super::ModifiersState>> {
  let conn = Connection::connect_to_env().unwrap();
  let (globals, event_queue) = registry_queue_init(&conn).unwrap();
  let qh = event_queue.handle();

  let mut event_loop: EventLoop<SyncToolState> =
    EventLoop::try_new().expect("Failed to initialize the event loop!");

  let loop_handle = event_loop.handle();
  WaylandSource::new(conn.clone(), event_queue)
    .insert(loop_handle)
    .unwrap();

  let mut state = SyncToolState {
    registry_state: RegistryState::new(&globals),
    seat_state: SeatState::new(&globals, &qh),
    keyboard: None,
    loop_handle: event_loop.handle(),
    modifiers: None,
  };

  event_loop
    .dispatch(Duration::from_millis(1000), &mut state)
    .unwrap();

  return Ok(state.modifiers);
}

impl SeatHandler for SyncToolState {
  fn seat_state(&mut self) -> &mut SeatState {
    &mut self.seat_state
  }

  fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {
    // Not applicable
  }

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
        .get_keyboard_with_repeat(
          qh,
          &seat,
          None,
          self.loop_handle.clone(),
          Box::new(|_state, _wl_kbd, event| {
            debug!("Repeat: {:?} ", event);
          }),
        )
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

  fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {
    // Not applicable
  }
}

impl ProvidesRegistryState for SyncToolState {
  fn registry(&mut self) -> &mut RegistryState {
    &mut self.registry_state
  }
  registry_handlers!(SyncToolState);
}

impl RegistryHandler<SyncToolState> for SyncToolState {}

impl KeyboardHandler for SyncToolState {
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
  }

  fn leave(
    &mut self,
    _: &Connection,
    _: &QueueHandle<Self>,
    _: &wl_keyboard::WlKeyboard,
    surface: &wl_surface::WlSurface,
    _: u32,
  ) {
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

delegate_registry!(SyncToolState);
delegate_seat!(SyncToolState);
delegate_keyboard!(SyncToolState);
