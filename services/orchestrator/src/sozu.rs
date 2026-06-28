use anyhow::Context;
use sozu_command_lib::{
    channel::Channel,
    proto::command::{
        AddBackend, IpAddress, RemoveBackend, Request,
        RequestHttpFrontend, RequestTcpFrontend, Response, ResponseStatus, SocketAddress,
        ip_address::Inner as IpInner, request::RequestType,
    },
};
use uuid::Uuid;

pub struct SozuClient {
    channel: Channel<Request, Response>,
    base_domain: String,
}

impl SozuClient {
    pub fn connect(socket_path: &str, base_domain: impl Into<String>) -> anyhow::Result<Self> {
        let channel = Channel::from_path(socket_path, 4096, 65536)
            .with_context(|| format!("connect to sozu socket {socket_path}"))?;
        Ok(Self {
            channel,
            base_domain: base_domain.into(),
        })
    }

    /// Register a workspace backend + HTTP frontend in sozu.
    /// Accessible at ws-<uid>.<base_domain> → container_ip:container_port
    pub fn add_workspace_route(
        &mut self,
        workspace_id: Uuid,
        workspace_uid: &str,
        container_ip: &str,
        container_port: u16,
    ) -> anyhow::Result<()> {
        let cluster_id = format!("ws-{workspace_id}");
        let backend_id = format!("ws-{workspace_id}-backend");
        let hostname = format!("{workspace_uid}.{}", self.base_domain);

        let addr = parse_socket_addr(container_ip, container_port)
            .with_context(|| format!("parse container address {container_ip}:{container_port}"))?;

        let backend = AddBackend {
            cluster_id: cluster_id.clone(),
            backend_id,
            address: addr,
            ..Default::default()
        };
        self.send(RequestType::AddBackend(backend))?;

        let frontend = RequestHttpFrontend {
            cluster_id: Some(cluster_id),
            address: make_socket_addr([0, 0, 0, 0], 80),
            hostname,
            ..Default::default()
        };
        self.send(RequestType::AddHttpFrontend(frontend))?;

        Ok(())
    }

    /// Remove a workspace route from sozu.
    pub fn remove_workspace_route(
        &mut self,
        workspace_id: Uuid,
        workspace_uid: &str,
        container_ip: &str,
        container_port: u16,
    ) -> anyhow::Result<()> {
        let cluster_id = format!("ws-{workspace_id}");
        let backend_id = format!("ws-{workspace_id}-backend");
        let hostname = format!("{workspace_uid}.{}", self.base_domain);

        let addr = parse_socket_addr(container_ip, container_port)
            .with_context(|| format!("parse container address {container_ip}:{container_port}"))?;

        self.send(RequestType::RemoveBackend(RemoveBackend {
            cluster_id: cluster_id.clone(),
            backend_id,
            address: addr,
        }))?;

        let frontend = RequestHttpFrontend {
            cluster_id: Some(cluster_id),
            address: make_socket_addr([0, 0, 0, 0], 80),
            hostname,
            ..Default::default()
        };
        self.send(RequestType::RemoveHttpFrontend(frontend))?;

        Ok(())
    }

    /// Register a TCP frontend in sozu for SSH access.
    /// Exposes container_ip:22 on exposed_port (range 2200–2999 for SSH, 5400–5499 for Postgres).
    pub fn add_workspace_tcp_route(
        &mut self,
        workspace_id: Uuid,
        container_ip: &str,
        container_port: u16,
        exposed_port: u16,
    ) -> anyhow::Result<()> {
        let cluster_id = format!("ws-tcp-{workspace_id}-{container_port}");
        let backend_id = format!("ws-tcp-{workspace_id}-{container_port}-backend");

        let addr = parse_socket_addr(container_ip, container_port)
            .with_context(|| format!("parse TCP address {container_ip}:{container_port}"))?;

        let backend = AddBackend {
            cluster_id: cluster_id.clone(),
            backend_id,
            address: addr,
            ..Default::default()
        };
        self.send(RequestType::AddBackend(backend))?;

        let frontend = RequestTcpFrontend {
            cluster_id,
            address: make_socket_addr([0, 0, 0, 0], exposed_port),
            ..Default::default()
        };
        self.send(RequestType::AddTcpFrontend(frontend))?;

        Ok(())
    }

    /// Remove a TCP frontend from sozu.
    pub fn remove_workspace_tcp_route(
        &mut self,
        workspace_id: Uuid,
        container_ip: &str,
        container_port: u16,
        exposed_port: u16,
    ) -> anyhow::Result<()> {
        let cluster_id = format!("ws-tcp-{workspace_id}-{container_port}");
        let backend_id = format!("ws-tcp-{workspace_id}-{container_port}-backend");

        let addr = parse_socket_addr(container_ip, container_port)
            .with_context(|| format!("parse TCP address {container_ip}:{container_port}"))?;

        self.send(RequestType::RemoveBackend(RemoveBackend {
            cluster_id: cluster_id.clone(),
            backend_id,
            address: addr,
        }))?;

        self.send(RequestType::RemoveTcpFrontend(RequestTcpFrontend {
            cluster_id,
            address: make_socket_addr([0, 0, 0, 0], exposed_port),
            ..Default::default()
        }))?;

        Ok(())
    }

    fn send(&mut self, request_type: RequestType) -> anyhow::Result<()> {
        let request = Request {
            request_type: Some(request_type),
        };
        self.channel
            .write_message(&request)
            .context("send sozu command")?;

        let response: Response = self
            .channel
            .read_message()
            .context("read sozu response")?;

        if response.status != ResponseStatus::Ok as i32 {
            anyhow::bail!("sozu error: status={} — {}", response.status, response.message);
        }
        Ok(())
    }
}

fn parse_socket_addr(ip_str: &str, port: u16) -> anyhow::Result<SocketAddress> {
    let ip: std::net::Ipv4Addr = ip_str.parse().context("parse IPv4")?;
    Ok(make_socket_addr(ip.octets(), port))
}

fn make_socket_addr(ip: [u8; 4], port: u16) -> SocketAddress {
    SocketAddress {
        ip: IpAddress {
            inner: Some(IpInner::V4(u32::from_be_bytes(ip))),
        },
        port: port as u32,
    }
}
