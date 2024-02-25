// This module was implemented starting from this wonderful example:
// https://github.com/Smithay/client-toolkit/blob/master/examples/kbd_input.rs

use std::cell::RefCell;
use std::cmp::min;
use std::rc::Rc;

use anyhow::{Context, Result};
use log::error;
use sctk::reexports::calloop;
use sctk::reexports::client::protocol::{wl_keyboard, wl_shm, wl_surface};
use sctk::seat::keyboard::{map_keyboard_repeat, Event as KbEvent, RepeatKind};
use sctk::shm::AutoMemPool;
use sctk::window::{Event as WEvent, FallbackFrame};

sctk::default_environment!(EspansoModifiersSync, desktop);

pub fn get_modifiers_state() -> Result<Option<super::ModifiersState>> {
  let (env, display, queue) = sctk::new_default_environment!(EspansoModifiersSync, desktop)
    .context("Unable to connect to a Wayland compositor")?;

  let result = Rc::new(RefCell::new(None));

  /*
   * Prepare a calloop event loop to handle key repetion
   */
  // Here `Option<WEvent>` is the type of a global value that will be shared by
  // all callbacks invoked by the event loop.
  let mut event_loop = calloop::EventLoop::<Option<WEvent>>::try_new().unwrap();

  /*
   * Create a buffer with window contents
   */

  let mut dimensions = (1u32, 1u32);

  /*
   * Init wayland objects
   */

  let surface = env.create_surface().detach();

  let mut window = env
    .create_window::<FallbackFrame, _>(surface, None, dimensions, move |evt, mut dispatch_data| {
      let next_action = dispatch_data.get::<Option<WEvent>>().unwrap();
      // Keep last event in priority order : Close > Configure > Refresh
      let replace = matches!(
        (&evt, &*next_action),
        (_, &None | &Some(WEvent::Refresh))
          | (&WEvent::Configure { .. }, &Some(WEvent::Configure { .. }))
          | (&WEvent::Close, _)
      );
      if replace {
        *next_action = Some(evt);
      }
    })
    .context("Failed to create a window !")?;

  window.set_title("Espanso Sync Tool".to_string());

  let mut pool = env
    .create_auto_pool()
    .context("Failed to create a memory pool !")?;

  /*
   * Keyboard initialization
   */

  let mut seats = Vec::<(String, Option<wl_keyboard::WlKeyboard>)>::new();

  // first process already existing seats
  for seat in env.get_all_seats() {
    if let Some((has_kbd, name)) = sctk::seat::with_seat_data(&seat, |seat_data| {
      (
        seat_data.has_keyboard && !seat_data.defunct,
        seat_data.name.clone(),
      )
    }) {
      if has_kbd {
        let result_clone = result.clone();
        match map_keyboard_repeat(
          event_loop.handle(),
          &seat,
          None,
          RepeatKind::System,
          move |event, _, _| keyboard_event_handler(event, &result_clone),
        ) {
          Ok(kbd) => {
            seats.push((name, Some(kbd)));
          }
          Err(e) => {
            error!("Failed to map keyboard on seat {} : {:?}.", name, e);
            seats.push((name, None));
          }
        }
      } else {
        seats.push((name, None));
      }
    }
  }

  // then setup a listener for changes
  let loop_handle = event_loop.handle();
  let result_clone = result.clone();
  let _seat_listener = env.listen_for_seats(move |seat, seat_data, _| {
    let result_clone = result_clone.clone();
    // find the seat in the vec of seats, or insert it if it is unknown
    let idx = seats.iter().position(|(name, _)| name == &seat_data.name);
    let idx = idx.unwrap_or_else(|| {
      seats.push((seat_data.name.clone(), None));
      seats.len() - 1
    });

    let (_, ref mut opt_kbd) = &mut seats[idx];
    // we should map a keyboard if the seat has the capability & is not defunct
    if seat_data.has_keyboard && !seat_data.defunct {
      if opt_kbd.is_none() {
        // we should initalize a keyboard
        match map_keyboard_repeat(
          loop_handle.clone(),
          &seat,
          None,
          RepeatKind::System,
          move |event, _, _| keyboard_event_handler(event, &result_clone),
        ) {
          Ok(kbd) => {
            *opt_kbd = Some(kbd);
          }
          Err(e) => {
            eprintln!(
              "Failed to map keyboard on seat {} : {:?}.",
              seat_data.name, e
            );
          }
        }
      }
    } else if let Some(kbd) = opt_kbd.take() {
      // the keyboard has been removed, cleanup
      kbd.release();
    }
  });

  if !env.get_shell().unwrap().needs_configure() {
    // initial draw to bootstrap on wl_shell
    redraw(&mut pool, window.surface(), dimensions).expect("Failed to draw");
    window.refresh();
  }

  sctk::WaylandSource::new(queue)
    .quick_insert(event_loop.handle())
    .unwrap();

  let mut next_action = None;

  loop {
    match next_action.take() {
      Some(WEvent::Close) => break,
      Some(WEvent::Refresh) => {
        window.refresh();
        window.surface().commit();
      }
      Some(WEvent::Configure {
        new_size,
        states: _,
      }) => {
        if let Some((w, h)) = new_size {
          window.resize(w, h);
          dimensions = (w, h);
        }
        window.refresh();
        redraw(&mut pool, window.surface(), dimensions).expect("Failed to draw");
      }
      None => {
        let result_clone = result.clone();
        let result_ref = result_clone.borrow();

        if let Some(result) = &*result_ref {
          return Ok(Some(*result));
        }
      }
    }

    // always flush the connection before going to sleep waiting for events
    display.flush().unwrap();

    event_loop
      .dispatch(Some(std::time::Duration::from_millis(10)), &mut next_action)
      .unwrap();
  }

  Ok(None)
}

fn keyboard_event_handler(
  event: KbEvent,
  result_clone: &Rc<RefCell<Option<super::ModifiersState>>>,
) {
  if let KbEvent::Modifiers { modifiers } = event {
    let mut result_mut = (**result_clone).borrow_mut();
    *result_mut = Some(super::ModifiersState {
      ctrl: modifiers.ctrl,
      alt: modifiers.alt,
      shift: modifiers.shift,
      caps_lock: modifiers.caps_lock,
      meta: modifiers.logo,
      num_lock: modifiers.num_lock,
    });
  }
}

#[allow(clippy::many_single_char_names)]
fn redraw(
  pool: &mut AutoMemPool,
  surface: &wl_surface::WlSurface,
  (buf_x, buf_y): (u32, u32),
) -> Result<(), ::std::io::Error> {
  let (canvas, new_buffer) = pool.buffer(
    buf_x as i32,
    buf_y as i32,
    4 * buf_x as i32,
    wl_shm::Format::Argb8888,
  )?;
  for (i, dst_pixel) in canvas.chunks_exact_mut(4).enumerate() {
    let x = i as u32 % buf_x;
    let y = i as u32 / buf_x;
    let r: u32 = min(((buf_x - x) * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
    let g: u32 = min((x * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
    let b: u32 = min(((buf_x - x) * 0xFF) / buf_x, (y * 0xFF) / buf_y);
    let pixel: [u8; 4] = ((0xFF << 24) + (r << 16) + (g << 8) + b).to_ne_bytes();
    dst_pixel[0] = pixel[0];
    dst_pixel[1] = pixel[1];
    dst_pixel[2] = pixel[2];
    dst_pixel[3] = pixel[3];
  }
  surface.attach(Some(&new_buffer), 0, 0);
  if surface.as_ref().version() >= 4 {
    surface.damage_buffer(0, 0, buf_x as i32, buf_y as i32);
  } else {
    surface.damage(0, 0, buf_x as i32, buf_y as i32);
  }
  surface.commit();
  Ok(())
}
