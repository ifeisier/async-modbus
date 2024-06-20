//! tcp 和 rtu 服务端 (Slaves).

use crate::Callback;
use anyhow::Result;
use std::io;
use std::net::SocketAddr;

/// 创建并启动新的 rtu 服务端
///
/// # 参数
/// - server_serial: 串口实例
///
/// # 返回
/// - 成功: 返 Server 实例
/// - 失败: 返回错误信息
#[cfg(feature = "modbus_rtu_server")]
pub async fn new_start_tru_server(
    server_serial: tokio_serial::SerialStream,
    slave_id: u8,
    on_call_back: Box<dyn Callback>,
) -> Result<()> {
    use crate::common_utils::InternalService;
    use std::sync::Arc;
    use tokio_modbus::server::rtu::Server;

    let server = Server::new(server_serial);

    let internal_service = Arc::new(InternalService {
        call_back: on_call_back,
        slave: slave_id,
    });

    server.serve_forever(internal_service).await?;
    Ok(())
}

/// 创建并启动新的 tcp 服务端
///
/// # 参数
/// - socket_addr: 监听的 ip 地址和端口
/// - slave_id: 从机 id
/// - on_call_back: 收到客户度消息后的回调
/// - on_process_error: 处理错误的回调
///
/// # 返回
/// - 成功: 返回空
/// - 失败: 返回错误信息
#[cfg(feature = "modbus_tcp_server")]
pub async fn new_start_tcp_server<OnProcessError>(
    socket_addr: SocketAddr,
    slave_id: u8,
    on_call_back: Box<dyn Callback>,
    on_process_error: OnProcessError,
) -> Result<()>
where
    OnProcessError: FnOnce(io::Error) + Clone + Send + 'static,
{
    use crate::common_utils::InternalService;
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio_modbus::server::tcp::{accept_tcp_connection, Server};

    let listener = TcpListener::bind(socket_addr).await?;
    let server = Server::new(listener);

    let internal_service = Arc::new(InternalService {
        call_back: on_call_back,
        slave: slave_id,
    });
    let new_service = |_socket_addr| Ok(Some(Arc::clone(&internal_service)));
    let on_connected = |stream, socket_addr| async move {
        accept_tcp_connection(stream, socket_addr, new_service)
    };

    server.serve(&on_connected, on_process_error).await?;
    Ok(())
}
