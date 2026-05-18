//! Tokio runtime en thread separado
//! Maneja la comunicación entre UI (egui) y core (tokio async)

use crate::bridge::{CoreEvent, GuiCommand};
use std::sync::mpsc::{channel, Sender, Receiver, RecvTimeoutError};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

pub struct AppRuntime {
    pub cmd_tx: Sender<GuiCommand>,
    pub event_rx: Receiver<CoreEvent>,
    _thread_handle: thread::JoinHandle<()>,
}

impl AppRuntime {
    /// Inicia el runtime en un thread separado
    pub fn start() -> Self {
        let (cmd_tx, cmd_rx) = channel::<GuiCommand>();
        let (event_tx, event_rx) = channel::<CoreEvent>();

        let thread_handle = thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create Tokio runtime");

            tracing::info!("AppRuntime started");

            loop {
                match cmd_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(cmd) => {
                        tracing::info!("AppRuntime received command: {:?}", cmd);

                        // Emitir un evento de vuelta como confirmación
                        let _ = event_tx.send(CoreEvent::LogLine(format!("Processed: {:?}", cmd)));

                        if let GuiCommand::Shutdown = cmd {
                            break;
                        }

                        // Aquí se podrían ejecutar operaciones async en `rt`:
                        // let event_tx = event_tx.clone();
                        // rt.block_on(async move {
                        //     // await en operaciones del core
                        // });
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        // No hay comandos pendientes — aquí podríamos hacer polling del estado
                        // por ejemplo cada 250ms emitir CoreEvent::QueueUpdated
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        tracing::info!("Command channel disconnected, shutting down runtime thread");
                        break;
                    }
                }
            }

            tracing::info!("AppRuntime shutdown");
        });

        AppRuntime {
            cmd_tx,
            event_rx,
            _thread_handle: thread_handle,
        }
    }

    /// Enviar un comando al runtime
    pub fn send_command(&self, cmd: GuiCommand) -> anyhow::Result<()> {
        self.cmd_tx.send(cmd)?;
        Ok(())
    }

    /// Drenar todos los eventos pendientes
    pub fn drain_events(&self) -> Vec<CoreEvent> {
        let mut events = Vec::new();
        loop {
            match self.event_rx.try_recv() {
                Ok(ev) => events.push(ev),
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
            }
        }
        events
    }
}

impl Drop for AppRuntime {
    fn drop(&mut self) {
        // Enviar shutdown al cerrar el App
        let _ = self.cmd_tx.send(GuiCommand::Shutdown);
    }
}
