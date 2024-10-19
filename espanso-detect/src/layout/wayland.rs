use nix::sys::socket;
use std::fs::File;
use std::io::Read;
use std::os::fd::BorrowedFd;
use wayland_client::Connection;
use wayland_client::Proxy;

fn pid_from_wayland_display_fd(fd: BorrowedFd) -> i32 {
  let soc_peer_process = socket::getsockopt(&fd, socket::sockopt::PeerCredentials).unwrap();
  soc_peer_process.pid()
}

fn get_compositor_pid() -> i32 {
  let wl_conn = Connection::connect_to_env().unwrap();
  let wl_display = wl_conn.display();
  let wl_display_backend = wl_display.backend().upgrade().unwrap();

  pid_from_wayland_display_fd(wl_display_backend.poll_fd())
}

fn process_name_from_pid(pid: i32) -> String {
  let path = format!("/proc/{pid}/comm");
  let mut file = File::open(path).unwrap();
  let mut buf = String::new();
  file.read_to_string(&mut buf).unwrap();
  buf
}
pub fn get_compositor_name() -> String {
  let pid = get_compositor_pid();
  process_name_from_pid(pid)
}
