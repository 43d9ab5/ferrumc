use async_trait::async_trait;
use futures::stream::FuturesUnordered;
use tracing::{debug_span, info, Instrument};

use crate::state::GlobalState;
use crate::utils::prelude::*;

mod chunk_sender;
mod keep_alive_system;
mod tick_system;
mod connection_handler;
mod command_system;


#[async_trait]
pub trait System: Send + Sync {
    async fn run(&self, state: GlobalState);
    fn name(&self) -> &'static str;
    async fn kill(&self) {}
}

pub static ALL_SYSTEMS: &[&dyn System] = &[
    &tick_system::TickSystem,
    &keep_alive_system::KeepAliveSystem,
    &chunk_sender::ChunkSender,
    &connection_handler::ConnectionHandler,
    &command_system::CommandSystem::new(),
];

pub async fn start_all_systems(state: GlobalState) -> Result<()> {
    let handles = FuturesUnordered::new();
    for system in ALL_SYSTEMS {
        let name = system.name();

        let handle = tokio::spawn(
            system
                .run(state.clone())
                .instrument(debug_span!("sys", %name)),
        );
        handles.push(handle);
    }

    futures::future::join_all(handles).await;

    Ok(())
}

pub async fn kill_all_systems() -> Result<()> {
    info!("Killing all systems...");
    for system in ALL_SYSTEMS {
        system.kill().await;
    }
    Ok(())
}
