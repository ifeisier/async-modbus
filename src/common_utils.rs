//! 公共模块

use crate::Callback;
use std::future;
use tokio_modbus::prelude::SlaveRequest;
use tokio_modbus::server::Service;
use tokio_modbus::{Exception, Request, Response};

/// 主要就是用来接收客户端的请求, 然后调用回调函数, 并将结果返回给客户端
pub(crate) struct InternalService {
    pub(crate) call_back: Box<dyn Callback>,
    pub(crate) slave: u8,
}

impl Service for InternalService {
    type Request = SlaveRequest<'static>;
    type Future = future::Ready<Result<Response, Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        if req.slave != self.slave {
            return future::ready(Err(Exception::IllegalDataAddress));
        }

        match req.request {
            Request::ReadCoils(address, cnt) => future::ready(
                self.call_back
                    .read_coils(address, cnt)
                    .map(Response::ReadCoils),
            ),
            Request::ReadDiscreteInputs(address, cnt) => future::ready(
                self.call_back
                    .read_discrete_inputs(address, cnt)
                    .map(Response::ReadDiscreteInputs),
            ),
            Request::WriteSingleCoil(address, cnt) => future::ready(
                self.call_back
                    .write_coil(address, cnt)
                    .map(|_| Response::WriteSingleCoil(address, cnt)),
            ),
            Request::WriteMultipleCoils(address, cnt) => future::ready(
                self.call_back
                    .write_coils(address, &cnt)
                    .map(|len| Response::WriteMultipleCoils(address, len)),
            ),
            Request::ReadHoldingRegisters(address, cnt) => future::ready(
                self.call_back
                    .read_holding_registers(address, cnt)
                    .map(Response::ReadHoldingRegisters),
            ),
            Request::ReadInputRegisters(address, cnt) => future::ready(
                self.call_back
                    .read_input_registers(address, cnt)
                    .map(Response::ReadInputRegisters),
            ),
            Request::WriteSingleRegister(address, value) => future::ready(
                self.call_back
                    .write_register(address, value)
                    .map(|_| Response::WriteSingleRegister(address, value)),
            ),
            Request::WriteMultipleRegisters(address, value) => future::ready(
                self.call_back
                    .write_registers(address, &value)
                    .map(|_| Response::WriteMultipleRegisters(address, value.len() as u16)),
            ),
            Request::MaskWriteRegister(address, and_mask, or_mask) => future::ready(
                self.call_back
                    .masked_write_register(address, and_mask, or_mask)
                    .map(|_| Response::MaskWriteRegister(address, and_mask, or_mask)),
            ),
            Request::ReadWriteMultipleRegisters(
                read_addr,
                read_count,
                write_addr,
                ref write_data,
            ) => future::ready(
                self.call_back
                    .read_write_multiple_registers(read_addr, read_count, write_addr, write_data)
                    .map(Response::ReadWriteMultipleRegisters),
            ),
            _ => {
                log::error!("SERVER: Exception::IllegalFunction - Unimplemented function code in request: {req:?}");
                future::ready(Err(Exception::IllegalFunction))
            }
        }
    }
}
