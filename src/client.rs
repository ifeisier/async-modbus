//! tcp 和 rtu 的通用客户端 (Master).
//!
//! tcp 和 rtu 客户端的使用方式是相同的, 所以通过 Client 同一实现, 并增加了超时重发功能.

use anyhow::{bail, Result};
use async_trait::async_trait;
use std::borrow::Cow;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::timeout;
use tokio_modbus::prelude::*;

enum ResultValue {
    U16(Vec<u16>),
    Bool(Vec<bool>),
}

/// tcp 和 rtu 客户端
pub struct Client {
    ctx: Box<client::Context>,
    timeout_millis: u64,
    retry_count: u64,
}

impl Client {
    /// 创建新的 Modbus TCP 协议客户端
    ///
    /// # 参数
    ///
    /// - socket_addr: socket 地址
    ///
    /// - slave_id: 从机 id
    ///
    /// # 返回
    ///
    /// - 成功: 返 Client 实例
    ///
    /// - 失败: 返回错误信息
    #[cfg(feature = "modbus_tcp_client")]
    pub async fn new_tcp(socket_addr: SocketAddr, slave_id: u8) -> Result<Client> {
        let ctx = tcp::connect_slave(socket_addr, Slave::from(slave_id))
            .await
            .unwrap();

        Ok(Client {
            ctx: Box::new(ctx),
            timeout_millis: 500,
            retry_count: 5,
        })
    }

    /// 创建新的 Modbus RTU 协议客户端
    ///
    /// # 参数
    ///
    /// - transport: 传输实例.
    ///
    ///   可以通过串口或者网络发送 Modbus RTU 协议.
    ///
    /// - slave_id: 从机 id
    ///
    /// # 返回
    ///
    /// - 成功: 返 Client 实例
    ///
    /// - 失败: 返回错误信息
    #[cfg(feature = "modbus_rtu_client")]
    pub async fn new_rtu<T>(transport: T, slave_id: u8) -> Result<Client>
    where
        T: AsyncRead + AsyncWrite + Debug + Unpin + Send + 'static,
    {
        let ctx = rtu::attach_slave(transport, Slave(slave_id));
        Ok(Client {
            ctx: Box::new(ctx),
            timeout_millis: 500,
            retry_count: 5,
        })
    }
}

#[async_trait]
impl crate::Writer for Client {
    async fn write_single_coil(&mut self, address: u16, value: bool) -> Result<()> {
        Ok(self
            .handle_timeout_write(Request::WriteSingleCoil(address, value))
            .await?)
    }

    async fn write_single_register(&mut self, address: u16, value: u16) -> Result<()> {
        Ok(self
            .handle_timeout_write(Request::WriteSingleRegister(address, value))
            .await?)
    }

    async fn write_multiple_coils(&mut self, address: u16, value: &[bool]) -> Result<()> {
        Ok(self
            .handle_timeout_write(Request::WriteMultipleCoils(address, Cow::from(value)))
            .await?)
    }

    async fn write_multiple_registers(&mut self, address: u16, value: &[u16]) -> Result<()> {
        Ok(self
            .handle_timeout_write(Request::WriteMultipleRegisters(address, Cow::from(value)))
            .await?)
    }

    async fn masked_write_register(
        &mut self,
        address: u16,
        and_mask: u16,
        or_mask: u16,
    ) -> Result<()> {
        Ok(self
            .handle_timeout_write(Request::MaskWriteRegister(address, and_mask, or_mask))
            .await?)
    }
}

#[async_trait]
impl crate::Reader for Client {
    async fn read_coils(&mut self, address: u16, count: u16) -> Result<Vec<bool>> {
        let result = self
            .handle_timeout_read(Request::ReadCoils(address, count))
            .await?;
        result_value_bool(result)
    }

    async fn read_discrete_inputs(&mut self, address: u16, count: u16) -> Result<Vec<bool>> {
        let result = self
            .handle_timeout_read(Request::ReadDiscreteInputs(address, count))
            .await?;
        result_value_bool(result)
    }

    async fn read_holding_registers(&mut self, address: u16, count: u16) -> Result<Vec<u16>> {
        let result = self
            .handle_timeout_read(Request::ReadHoldingRegisters(address, count))
            .await?;
        result_value_u16(result)
    }

    async fn read_input_registers(&mut self, address: u16, count: u16) -> Result<Vec<u16>> {
        let result = self
            .handle_timeout_read(Request::ReadInputRegisters(address, count))
            .await?;
        result_value_u16(result)
    }

    async fn read_write_multiple_registers(
        &mut self,
        read_addr: u16,
        read_count: u16,
        write_addr: u16,
        write_data: &[u16],
    ) -> Result<Vec<u16>> {
        let result = self
            .handle_timeout_read(Request::ReadWriteMultipleRegisters(
                read_addr,
                read_count,
                write_addr,
                Cow::from(write_data),
            ))
            .await?;
        result_value_u16(result)
    }
}

impl Client {
    /// 写超时后会重试
    async fn handle_timeout_write(&mut self, request: Request<'_>) -> Result<()> {
        let mut retry_count = self.retry_count;
        let timeout_duration = Duration::from_millis(self.timeout_millis);

        while retry_count > 0 {
            let future = match request {
                Request::WriteSingleCoil(address, coil) => {
                    self.ctx.write_single_coil(address, coil)
                }
                Request::WriteSingleRegister(address, data) => {
                    self.ctx.write_single_register(address, data)
                }
                Request::WriteMultipleCoils(address, ref coil) => {
                    self.ctx.write_multiple_coils(address, coil)
                }
                Request::WriteMultipleRegisters(address, ref coil) => {
                    self.ctx.write_multiple_registers(address, coil)
                }
                Request::MaskWriteRegister(address, and_mask, or_mask) => {
                    self.ctx.masked_write_register(address, and_mask, or_mask)
                }
                _ => {
                    bail!("Out of handle_timeout options range")
                }
            };

            match timeout(timeout_duration, future).await {
                Ok(Ok(response)) => {
                    return Ok(response);
                }
                Ok(Err(e)) => {
                    bail!(e)
                }
                Err(_) => {
                    retry_count -= 1;
                    continue;
                }
            }
        }
        bail!("Timeout: deadline has elapsed")
    }

    /// 处理读超时
    async fn handle_timeout_read(&mut self, request: Request<'_>) -> Result<ResultValue> {
        let timeout_duration = Duration::from_millis(self.timeout_millis);
        let mut retry_count = self.retry_count;

        while retry_count > 0 {
            let future =
                match request {
                    Request::ReadHoldingRegisters(addr, cnt) => {
                        Some(self.ctx.read_holding_registers(addr, cnt))
                    }
                    Request::ReadInputRegisters(addr, cnt) => {
                        Some(self.ctx.read_input_registers(addr, cnt))
                    }
                    Request::ReadWriteMultipleRegisters(
                        read_addr,
                        read_count,
                        write_addr,
                        ref write_data,
                    ) => Some(self.ctx.read_write_multiple_registers(
                        read_addr, read_count, write_addr, write_data,
                    )),
                    _ => None,
                };
            if let Some(future) = future {
                match timeout(timeout_duration, future).await {
                    Ok(Ok(response)) => {
                        return Ok(ResultValue::U16(response));
                    }
                    Ok(Err(e)) => {
                        bail!(e)
                    }
                    Err(_) => {
                        retry_count -= 1;
                        continue;
                    }
                }
            }
            drop(future);

            let future = match request {
                Request::ReadCoils(addr, cnt) => Some(self.ctx.read_coils(addr, cnt)),
                Request::ReadDiscreteInputs(addr, cnt) => {
                    Some(self.ctx.read_discrete_inputs(addr, cnt))
                }
                _ => None,
            };
            if let Some(future) = future {
                match timeout(timeout_duration, future).await {
                    Ok(Ok(response)) => {
                        return Ok(ResultValue::Bool(response));
                    }
                    Ok(Err(e)) => {
                        bail!(e)
                    }
                    Err(_) => {
                        retry_count -= 1;
                    }
                }
            } else {
                bail!("Out of handle_timeout options range")
            }
        }
        bail!("Timeout: deadline has elapsed")
    }
}

fn result_value_bool(result: ResultValue) -> Result<Vec<bool>> {
    match result {
        ResultValue::Bool(v) => Ok(v),
        _ => {
            bail!("Result is not bool")
        }
    }
}

fn result_value_u16(result: ResultValue) -> Result<Vec<u16>> {
    match result {
        ResultValue::U16(v) => Ok(v),
        _ => {
            bail!("Result is not u16")
        }
    }
}
