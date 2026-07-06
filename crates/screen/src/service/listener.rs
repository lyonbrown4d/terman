use std::{
    io,
    sync::{Arc, Mutex, mpsc},
    thread,
};

use interprocess::local_socket::prelude::*;

use super::listener_dispatch::handle_client;
use crate::{
    ipc::ScreenIpcEndpoint,
    session_core::{ScreenControlEvent, ScreenSessionBus},
};

pub(crate) struct ScreenSessionService {
    _handle: thread::JoinHandle<()>,
}

impl ScreenSessionService {
    pub(crate) fn start(
        session_name: Option<Arc<Mutex<String>>>,
        endpoint: ScreenIpcEndpoint,
        bus: ScreenSessionBus,
        control_tx: mpsc::Sender<ScreenControlEvent>,
    ) -> io::Result<Option<Self>> {
        let Some(session_name) = session_name else {
            return Ok(None);
        };

        let listener = endpoint.listener_options()?.create_sync()?;
        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else {
                    continue;
                };
                let client_bus = bus.clone();
                let client_control_tx = control_tx.clone();
                let client_session_name = session_name.clone();
                thread::spawn(move || {
                    let _ = handle_client(
                        &mut stream,
                        &client_session_name,
                        &client_bus,
                        &client_control_tx,
                    );
                });
            }
        });

        Ok(Some(Self { _handle: handle }))
    }
}