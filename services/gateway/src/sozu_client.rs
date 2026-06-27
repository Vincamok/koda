use std::net::SocketAddr;

use anyhow::Context;
use sozu_command_lib::{
    channel::Channel,
    command::{CommandRequest, CommandRequestOrder, CommandResponse, CommandStatus},
    proxy::{Backend, HttpFrontend, HttpFrontendOrder, Route},
};
use uuid::Uuid;

pub struct SozuClient {
    channel: Channel<CommandRequest, CommandResponse>,
    base_domain: String,
}

impl SozuClient {
    pub fn connect(socket_path: &str, base_domain: impl Into<String>) -> anyhow::Result<Self> {
        let channel = Channel::from_path(socket_path, 4096, 4096)
            .with_context(|| format!("connect to sozu socket {socket_path}"))?;
        Ok(Self {
            channel,
            base_domain: base_domain.into(),
        })
    }

    /// Register a workspace backend + frontend in sozu.
    /// The workspace is accessible at ws-<uid>.<base_domain>
    pub fn add_workspace_route(
        &mut self,
        workspace_id: Uuid,
        workspace_uid: &str,
        container_port: u16,
    ) -> anyhow::Result<()> {
        let cluster_id = format!("ws-{}", workspace_id);
        let backend_id = format!("ws-{}-backend", workspace_id);
        let hostname = format!("{}.{}", workspace_uid, self.base_domain);

        // Add backend
        let backend = Backend {
            cluster_id: cluster_id.clone(),
            backend_id: backend_id.clone(),
            address: SocketAddr::from(([127, 0, 0, 1], container_port)),
            ..Default::default()
        };
        self.send(CommandRequestOrder::AddBackend(backend))?;

        // Add HTTP frontend
        let frontend = HttpFrontend {
            route: Route::ClusterId(cluster_id),
            address: "0.0.0.0:80".parse().unwrap(),
            hostname,
            ..Default::default()
        };
        self.send(CommandRequestOrder::AddHttpFrontend(frontend))?;

        Ok(())
    }

    /// Remove a workspace route from sozu.
    pub fn remove_workspace_route(
        &mut self,
        workspace_id: Uuid,
        workspace_uid: &str,
        container_port: u16,
    ) -> anyhow::Result<()> {
        let cluster_id = format!("ws-{}", workspace_id);
        let backend_id = format!("ws-{}-backend", workspace_id);
        let hostname = format!("{}.{}", workspace_uid, self.base_domain);

        let backend = Backend {
            cluster_id: cluster_id.clone(),
            backend_id,
            address: SocketAddr::from(([127, 0, 0, 1], container_port)),
            ..Default::default()
        };
        self.send(CommandRequestOrder::RemoveBackend(backend))?;

        let frontend = HttpFrontend {
            route: Route::ClusterId(cluster_id),
            address: "0.0.0.0:80".parse().unwrap(),
            hostname,
            ..Default::default()
        };
        self.send(CommandRequestOrder::RemoveHttpFrontend(frontend))?;

        Ok(())
    }

    fn send(&mut self, order: CommandRequestOrder) -> anyhow::Result<()> {
        let request = CommandRequest {
            id: Uuid::new_v4().to_string(),
            version: 0,
            worker_id: None,
            order,
        };
        self.channel.write_message(&request).context("send sozu command")?;

        // Read response
        let response: CommandResponse = self.channel.read_message().context("read sozu response")?;
        if response.status != CommandStatus::Ok {
            anyhow::bail!("sozu error: {:?} — {:?}", response.status, response.message);
        }
        Ok(())
    }
}
