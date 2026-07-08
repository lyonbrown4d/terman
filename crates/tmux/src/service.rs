use std::{
    io,
    sync::{Arc, Mutex, mpsc},
    thread,
};

use interprocess::local_socket::traits::ListenerExt;

use crate::{
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service_codec::{read_response, write_request},
    service_handlers::handle_client,
    session_core::{TmuxControlEvent, TmuxSessionBus},
};

#[allow(dead_code)]
pub(crate) struct TmuxSessionService {
    _handle: thread::JoinHandle<()>,
}

impl TmuxSessionService {
    #[allow(dead_code)]
    pub(crate) fn start(
        session_name: Arc<Mutex<String>>,
        endpoint: TmuxIpcEndpoint,
        cwd: String,
        bus: TmuxSessionBus,
        control_tx: mpsc::Sender<TmuxControlEvent>,
    ) -> io::Result<Self> {
        let listener = endpoint.listener_options()?.create_sync()?;
        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue; };
                let client_bus = bus.clone();
                let client_control_tx = control_tx.clone();
                let client_session_name = session_name.clone();
                let client_cwd = cwd.clone();
                thread::spawn(move || {
                    let _ = handle_client(
                        &mut stream,
                        &client_session_name,
                        &client_cwd,
                        &client_bus,
                        &client_control_tx,
                    );
                });
            }
        });

        Ok(Self { _handle: handle })
    }
}

#[allow(dead_code)]
pub(crate) fn request_endpoint_response(
    endpoint: &TmuxIpcEndpoint,
    request: TmuxIpcRequest,
) -> io::Result<TmuxIpcResponse> {
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    write_request(&mut stream, &request)?;
    read_response(stream)
}

#[cfg(test)]
mod tests {
    use crate::ipc::TmuxIpcRequest;

    #[test]
    fn models_client_request_payload() {
        assert_eq!(TmuxIpcRequest::Ping, TmuxIpcRequest::Ping);
    }
}