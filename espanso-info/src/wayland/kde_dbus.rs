/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

//! Native D-Bus client for `KWin`'s scripting interface, used to query the
//! active window on KDE Plasma (both Wayland and X11 sessions) without
//! external tools.
//!
//! `KWin` has no direct "get active window" D-Bus method, but it can load and
//! run scripts (`org.kde.KWin` service, `/Scripting` object). The loaded
//! script runs inside the compositor, reads `workspace.activeWindow`, and
//! reports back by *calling us over D-Bus* (`KWin`'s `callDBus()` JS function,
//! addressed at this connection's unique bus name) — necessary because
//! `org.kde.kwin.Script.run()` has no return value.
//!
//! Prior art: kdotool (<https://github.com/jinliu/kdotool>) uses the same
//! `KWin` interface; this module replaces espanso's previous dependency on the
//! kdotool binary.

use std::sync::mpsc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};

/// `KWin` JS executed inside the compositor. Placeholders: `{{dbus_addr}}` is
/// our unique bus name (e.g. `:1.42`), `{{seq}}` a per-query sequence number
/// used to discard stale replies from earlier timed-out queries.
const KWIN_SCRIPT: &str = r#"
function output_result(message) {
    callDBus("{{dbus_addr}}", "/", "", "result", message.toString());
}
function output_error(message) {
    callDBus("{{dbus_addr}}", "/", "", "error", message.toString());
}
var w = workspace.activeWindow;
if (w == null) {
    output_error("No active window");
} else {
    output_result(JSON.stringify({
        seq: {{seq}},
        title: w.caption,
        class_name: w.resourceClass,
        pid: w.pid
    }));
}
"#;

const KWIN_SERVICE: &str = "org.kde.KWin";
const SCRIPTING_PATH: &str = "/Scripting";
const SCRIPTING_INTERFACE: &str = "org.kde.kwin.Scripting";
const SCRIPT_INTERFACE: &str = "org.kde.kwin.Script";
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub(crate) struct KdeWindowInfo {
    pub title: Option<String>,
    pub class_name: Option<String>,
    pub pid: Option<i64>,
}

/// Returns true if `KWin`'s D-Bus service is reachable on the session bus.
pub(crate) fn is_kwin_available() -> bool {
    let Ok(conn) = zbus::blocking::Connection::session() else {
        return false;
    };
    let Ok(proxy) = zbus::blocking::fdo::DBusProxy::new(&conn) else {
        return false;
    };
    match KWIN_SERVICE.try_into() {
        Ok(name) => proxy.name_has_owner(name).unwrap_or(false),
        Err(_) => false,
    }
}

pub(crate) struct KwinScriptingClient {
    conn: zbus::blocking::Connection,
    dbus_addr: String,
    rx: mpsc::Receiver<(String, String)>,
    seq: std::cell::Cell<u64>,
}

impl KwinScriptingClient {
    pub fn new() -> Result<Self> {
        let conn =
            zbus::blocking::Connection::session().context("cannot connect to session bus")?;
        let dbus_addr = conn
            .unique_name()
            .context("session connection has no unique name")?
            .to_string();

        // The KWin script addresses us directly by unique name with an empty
        // interface, so nothing is registered on an ObjectServer: a dedicated
        // thread reads the raw method calls off the message stream and
        // forwards them through a channel. It lives as long as the
        // connection, and exits once the receiver side is dropped (the send
        // fails on the next incoming message).
        let iter = zbus::blocking::MessageIterator::from(conn.clone());
        let (tx, rx) = mpsc::channel::<(String, String)>();
        std::thread::Builder::new()
            .name("espanso-kwin-dbus-callback".to_string())
            .spawn(move || {
                for msg in iter.flatten() {
                    let header = msg.header();
                    if header.message_type() != zbus::message::Type::MethodCall {
                        continue;
                    }
                    let Some(member) = header.member() else {
                        continue;
                    };
                    let member = member.to_string();
                    if member != "result" && member != "error" {
                        continue;
                    }
                    let Ok(payload) = msg.body().deserialize::<String>() else {
                        continue;
                    };
                    if tx.send((member, payload)).is_err() {
                        break;
                    }
                }
            })
            .context("cannot spawn D-Bus callback thread")?;

        Ok(Self {
            conn,
            dbus_addr,
            rx,
            seq: std::cell::Cell::new(0),
        })
    }

    pub fn get_active_window_info(&self) -> Result<KdeWindowInfo> {
        let seq = self.seq.get().wrapping_add(1);
        self.seq.set(seq);

        // Drop replies from previous queries that arrived after their timeout.
        while self.rx.try_recv().is_ok() {}

        let script = KWIN_SCRIPT
            .replace("{{dbus_addr}}", &self.dbus_addr)
            .replace("{{seq}}", &seq.to_string());
        let script_name = format!("espanso-window-query-{}", std::process::id());
        let script_path = std::env::temp_dir().join(format!("{script_name}.js"));
        std::fs::write(&script_path, script).context("cannot write KWin script temp file")?;

        let result = self.run_script(&script_path, &script_name, seq);
        let _ = std::fs::remove_file(&script_path);
        result
    }

    fn run_script(
        &self,
        script_path: &std::path::Path,
        script_name: &str,
        seq: u64,
    ) -> Result<KdeWindowInfo> {
        let script_path_str = script_path.to_str().context("non-UTF8 temp path")?;

        // A leftover script with the same name (e.g. from a crashed run)
        // makes loadScript fail, so unload defensively first.
        let _ = self.unload_script(script_name);

        let reply = self
            .conn
            .call_method(
                Some(KWIN_SERVICE),
                SCRIPTING_PATH,
                Some(SCRIPTING_INTERFACE),
                "loadScript",
                &(script_path_str, script_name),
            )
            .context("loadScript failed — is KWin running on this session bus?")?;
        let script_id: i32 = reply.body().deserialize().context("bad loadScript reply")?;
        if script_id < 0 {
            bail!("KWin refused to load the script (id = {})", script_id);
        }

        let script_obj = format!("{SCRIPTING_PATH}/Script{script_id}");
        let outcome = self
            .conn
            .call_method(
                Some(KWIN_SERVICE),
                script_obj.as_str(),
                Some(SCRIPT_INTERFACE),
                "run",
                &(),
            )
            .context("Script.run() failed")
            .and_then(|_| self.wait_for_reply(seq));

        // Best-effort cleanup regardless of outcome.
        let _ = self.conn.call_method(
            Some(KWIN_SERVICE),
            script_obj.as_str(),
            Some(SCRIPT_INTERFACE),
            "stop",
            &(),
        );
        let _ = self.unload_script(script_name);

        outcome
    }

    fn wait_for_reply(&self, seq: u64) -> Result<KdeWindowInfo> {
        let deadline = std::time::Instant::now() + CALLBACK_TIMEOUT;
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            match self.rx.recv_timeout(remaining) {
                Ok((member, payload)) if member == "result" => {
                    let parsed: serde_json::Value = serde_json::from_str(&payload)
                        .context("KWin callback payload is not valid JSON")?;
                    if parsed["seq"].as_u64() != Some(seq) {
                        // Stale reply from an earlier timed-out query.
                        continue;
                    }
                    return Ok(KdeWindowInfo {
                        title: parsed["title"].as_str().map(str::to_string),
                        class_name: parsed["class_name"].as_str().map(str::to_string),
                        pid: parsed["pid"].as_i64(),
                    });
                }
                Ok((_, payload)) => return Err(anyhow!("KWin script error: {}", payload)),
                Err(_) => {
                    return Err(anyhow!(
                        "timed out after {:?} waiting for the KWin callback",
                        CALLBACK_TIMEOUT
                    ))
                }
            }
        }
    }

    fn unload_script(&self, script_name: &str) -> zbus::Result<zbus::message::Message> {
        self.conn.call_method(
            Some(KWIN_SERVICE),
            SCRIPTING_PATH,
            Some(SCRIPTING_INTERFACE),
            "unloadScript",
            &(script_name,),
        )
    }
}
