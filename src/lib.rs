#![warn(missing_docs)]
//! Modbus 通信采用[主从 (Master-Slave)](https://en.wikipedia.org/wiki/Master%E2%80%93slave_(technology)) 架构.
//!
//! 一个 Modbus 网络上通常有一个主站 (Master) 和一个或多个从站 (Slaves), 主站连接到从站并发起通信, 而从站只是负责响应.
//!
//! Modbus 协议有多个版本, 包括:
//! - Modbus RTU: 在 RS-485 或 RS-232 串行通信上实现, 使用二进制编码, 效率高.
//! - Modbus ASCII: 在 RS-485 或 RS-232 串行通信上实现, 使用 ASCII 字符编码, 便于调试.
//! - Modbus TCP: 在以太网上实现, 使用 TCP/IP 协议进行传输, 适用于现代网络.
//!
//! 数据模型: Modbus 使用一种简单的数据模型, 包括四种主要数据类型:
//! - 离散输入(Discrete Input): 单个位的只读数据.
//! - 线圈(Coil): 单个位的读写数据.
//! - 输入寄存器(Input Register): 16 位的只读数据.
//! - 保持寄存器(Holding Register): 16 位的读写数据.

use anyhow::Result;
use async_trait::async_trait;

#[cfg(any(feature = "modbus_tcp_client", feature = "modbus_rtu_client"))]
pub mod client;

#[cfg(any(feature = "modbus_tcp_server", feature = "modbus_rtu_server"))]
pub mod server;

#[cfg(any(feature = "modbus_tcp_server", feature = "modbus_rtu_server"))]
mod common_utils;

pub use tokio_modbus::Exception;

/// 异步读 Modbus 数据
#[async_trait]
pub trait Reader {
    /// 读取多个线圈 (0x01)
    ///
    /// # 参数
    /// - address: 要读取的第一个起始地址
    /// - count: 从地址 `address` 开始读取的数量
    ///
    /// # 返回
    /// - 成功: 返回读取的数据
    /// - 失败: 返回错误信息
    async fn read_coils(&mut self, address: u16, count: u16) -> Result<Vec<bool>>;

    /// 读取多个离散输入 (0x02)
    ///
    /// # 参数
    /// - address: 要读取的第一个起始地址
    /// - count: 从地址 `address` 开始读取的数量
    ///
    /// # 返回
    /// - 成功: 返回读取的数据
    /// - 失败: 返回错误信息
    async fn read_discrete_inputs(&mut self, address: u16, count: u16) -> Result<Vec<bool>>;

    /// 读取多个保持寄存器 (0x03)
    ///
    /// # 参数
    /// - address: 要读取的第一个起始地址
    /// - count: 从地址 `address` 开始读取的数量
    ///
    /// # 返回
    /// - 成功: 返回读取的数据
    /// - 失败: 返回错误信息
    async fn read_holding_registers(&mut self, address: u16, count: u16) -> Result<Vec<u16>>;

    /// 读取多个输入寄存器 (0x04)
    ///
    /// # 参数
    /// - address: 要读取的第一个起始地址
    /// - count: 从地址 `address` 开始读取的数量
    ///
    /// # 返回
    /// - 成功: 返回读取的数据
    /// - 失败: 返回错误信息
    async fn read_input_registers(&mut self, address: u16, count: u16) -> Result<Vec<u16>>;

    /// 读取和写入多个保持寄存器 (0x17)
    ///
    /// # 参数
    /// - read_addr: 读地址
    /// - read_count: 读数量
    /// - write_addr: 写地址
    /// - write_data: 写数据
    ///
    /// # 返回
    /// - 成功: 返回读取的数据
    /// - 失败: 返回错误信息
    async fn read_write_multiple_registers(
        &mut self,
        read_addr: u16,
        read_count: u16,
        write_addr: u16,
        write_data: &[u16],
    ) -> Result<Vec<u16>>;
}

/// 异步写 Modbus 数据
#[async_trait]
pub trait Writer {
    /// 写入单个线圈 (0x05)
    ///
    /// # 参数
    /// - address: 要写入的地址
    /// - value: 要写入的值
    ///
    /// # 返回
    /// - 成功: 返回空
    /// - 失败: 返回错误信息
    async fn write_single_coil(&mut self, address: u16, value: bool) -> Result<()>;

    /// 写入单个保持寄存器 (0x06)
    ///
    /// # 参数
    /// - address: 要写入的地址
    /// - value: 要写入的值
    ///
    /// # 返回
    /// - 成功: 返回空
    /// - 失败: 返回错误信息
    async fn write_single_register(&mut self, address: u16, value: u16) -> Result<()>;

    /// 写入多个线圈 (0x0F)
    ///
    /// # 参数
    /// - address: 要写入的第一个起始地址
    /// - value: 从地址 `address` 开始写入的值
    ///
    /// # 返回
    /// - 成功: 返回空
    /// - 失败: 返回错误信息
    async fn write_multiple_coils(&mut self, address: u16, value: &[bool]) -> Result<()>;

    /// 写入多个保持寄存器 (0x10)
    ///
    /// # 参数
    /// - address: 要写入的第一个起始地址
    /// - value: 从地址 `address` 开始写入的值
    ///
    /// # 返回
    /// - 成功: 返回空
    /// - 失败: 返回错误信息
    async fn write_multiple_registers(&mut self, address: u16, value: &[u16]) -> Result<()>;

    /// 设置或清除单个保持寄存器的位 (0x16)
    ///
    /// # 参数
    /// - address: 地址
    /// - and_mask: AND 掩码
    /// - or_mask: OR 掩码
    ///
    /// # 返回
    /// - 成功: 返回空
    /// - 失败: 返回错误信息
    async fn masked_write_register(
        &mut self,
        address: u16,
        and_mask: u16,
        or_mask: u16,
    ) -> Result<()>;
}

#[cfg(any(feature = "modbus_tcp_server", feature = "modbus_rtu_server",))]
/// 收到客户端消息的回调接口
pub trait Callback: Send + Sync + 'static {
    /// 读取多个线圈 (0x01)
    ///
    /// # 参数
    /// - address: 要读取的第一个起始地址
    /// - count: 从地址 `address` 开始读取的数量
    ///
    /// # 返回
    /// - 成功: 返回读取的数据
    /// - 失败: 返回错误信息
    fn read_coils(
        &self,
        address: u16,
        count: u16,
    ) -> std::result::Result<Vec<bool>, tokio_modbus::Exception>;

    /// 读取多个离散输入 (0x02)
    ///
    /// # 参数
    /// - address: 要读取的第一个起始地址
    /// - count: 从地址 `address` 开始读取的数量
    ///
    /// # 返回
    /// - 成功: 返回读取的数据
    /// - 失败: 返回错误信息
    fn read_discrete_inputs(
        &self,
        address: u16,
        count: u16,
    ) -> std::result::Result<Vec<bool>, tokio_modbus::Exception>;

    /// 写入单个线圈 (0x05)
    ///
    /// # 参数
    /// - address: 要写入的地址
    /// - value: 要写入的值
    ///
    /// # 返回
    /// - 成功: 返回写入的值
    /// - 失败: 返回错误信息
    fn write_coil(
        &self,
        address: u16,
        value: bool,
    ) -> std::result::Result<bool, tokio_modbus::Exception>;

    /// 写入多个线圈 (0x0F)
    ///
    /// # 参数
    /// - address: 要写入的第一个起始地址
    /// - value: 从地址 `address` 开始写入的值
    ///
    /// # 返回
    /// - 成功: 返回写入的长度
    /// - 失败: 返回错误信息
    fn write_coils(
        &self,
        address: u16,
        values: &[bool],
    ) -> std::result::Result<u16, tokio_modbus::Exception>;

    /// 读取多个保持寄存器 (0x03)
    ///
    /// # 参数
    /// - address: 要读取的第一个起始地址
    /// - count: 从地址 `address` 开始读取的数量
    ///
    /// # 返回
    /// - 成功: 返回读取的数据
    /// - 失败: 返回错误信息
    fn read_holding_registers(
        &self,
        address: u16,
        count: u16,
    ) -> std::result::Result<Vec<u16>, tokio_modbus::Exception>;

    /// 读取多个输入寄存器 (0x04)
    ///
    /// # 参数
    /// - address: 要读取的第一个起始地址
    /// - count: 从地址 `address` 开始读取的数量
    ///
    /// # 返回
    /// - 成功: 返回读取的数据
    /// - 失败: 返回错误信息
    fn read_input_registers(
        &self,
        address: u16,
        count: u16,
    ) -> std::result::Result<Vec<u16>, tokio_modbus::Exception>;

    /// 写入单个保持寄存器 (0x06)
    ///
    /// # 参数
    /// - address: 要写入的地址
    /// - value: 要写入的值
    ///
    /// # 返回
    /// - 成功: 返回写入的值
    /// - 失败: 返回错误信息
    fn write_register(
        &self,
        address: u16,
        value: u16,
    ) -> std::result::Result<u16, tokio_modbus::Exception>;

    /// 写入多个保持寄存器 (0x10)
    ///
    /// # 参数
    /// - address: 要写入的第一个起始地址
    /// - value: 从地址 `address` 开始写入的值
    ///
    /// # 返回
    /// - 成功: 返回写入的长度
    /// - 失败: 返回错误信息
    fn write_registers(
        &self,
        address: u16,
        value: &[u16],
    ) -> std::result::Result<u16, tokio_modbus::Exception>;

    /// 设置或清除单个保持寄存器的位 (0x16)
    ///
    /// # 参数
    /// - address: 地址
    /// - and_mask: AND 掩码
    /// - or_mask: OR 掩码
    ///
    /// # 返回
    /// - 成功: 返回空
    /// - 失败: 返回错误信息
    fn masked_write_register(
        &self,
        address: u16,
        and_mask: u16,
        or_mask: u16,
    ) -> std::result::Result<(), tokio_modbus::Exception>;

    /// 读取和写入多个保持寄存器 (0x17)
    ///
    /// # 参数
    /// - read_addr: 读地址
    /// - read_count: 读数量
    /// - write_addr: 写地址
    /// - write_data: 写数据
    ///
    /// # 返回
    /// - 成功: 返回读取的数据
    /// - 失败: 返回错误信息
    fn read_write_multiple_registers(
        &self,
        read_addr: u16,
        read_count: u16,
        write_addr: u16,
        write_data: &[u16],
    ) -> std::result::Result<Vec<u16>, tokio_modbus::Exception>;
}
