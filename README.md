# async-modbus

在 tokio-modbus 的基础上增加了超时重发功能.


## 使用 modbus_tcp_client

```rust
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};


fn main() {
   let runtime = new_current_thread().unwrap();
   runtime.block_on(async {
      use async_modbus::client::Client;
      use async_modbus::Reader;

      let socket_addr = "192.168.200.153:502".parse().unwrap();
      let mut client = Client::new_tcp(socket_addr, 1).await.unwrap();
      println!("{:?}", client.read_coils(0, 1).await.unwrap());
   });
}
```


## 使用 modbus_tcp_server

```rust
use async_modbus::{Callback, Exception};
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};

struct TempCallback;

impl Callback for TempCallback {
   fn read_coils(&self, address: u16, count: u16) -> Result<Vec<bool>, Exception> {
      println!("read_coils: {}, {}", address, count);
      Ok(vec![true])
   }

   // ...
}

fn main() {
   let runtime = new_current_thread().unwrap();
   runtime.block_on(async {
      use async_modbus::server;

      let socket_addr = "0.0.0.0:8080".parse().unwrap();
      server::new_start_tcp_server(socket_addr, 1, Box::new(TempCallback), |error| todo!())
              .await
              .unwrap();
   });
}
```
