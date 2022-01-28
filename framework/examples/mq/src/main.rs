/*
 * Copyright 2022. the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::collections::HashMap;
use std::env;
use std::time::Duration;

use testcontainers::clients;
use tokio::time::sleep;

use bios::basic::config::NoneConfig;
use bios::basic::result::BIOSResult;
use bios::test::test_container::BIOSTestContainer;
use bios::BIOSFuns;

#[tokio::main]
async fn main() -> BIOSResult<()> {
    // Here is a demonstration of using docker to start a mysql simulation scenario.
    let docker = clients::Cli::default();
    let rabbit_container = BIOSTestContainer::rabbit_custom(&docker);
    let port = rabbit_container.get_host_port(5672).expect("Test port acquisition error");
    let url = format!("amqp://guest:guest@127.0.0.1:{}/%2f", port);
    env::set_var("BIOS_MQ.URL", url);

    env::set_var("RUST_LOG", "debug");
    env::set_var("PROFILE", "default");

    // Initial configuration
    BIOSFuns::init::<NoneConfig>("config").await?;

    let client = BIOSFuns::mq();

    // --------------------------------------------------

    let mut header = HashMap::new();
    header.insert("k1".to_string(), "v1".to_string());

    /*let latch_req = CountDownLatch::new(4);
    let latch_cp = latch_req.clone();*/
    client
        .response("test-addr", |(header, msg)| async move {
            println!("response1");
            assert_eq!(header.get("k1").unwrap(), "v1");
            assert_eq!(msg, "测试!");
            // move occurs because ..., which does not implement the `Copy` trait
            //latch_cp.countdown();
            Ok(())
        })
        .await?;

    client
        .response("test-addr", |(header, msg)| async move {
            println!("response2");
            assert_eq!(header.get("k1").unwrap(), "v1");
            assert_eq!(msg, "测试!");
            Ok(())
        })
        .await?;

    client.request("test-addr", "测试!".to_string(), &header).await?;
    client.request("test-addr", "测试!".to_string(), &header).await?;
    client.request("test-addr", "测试!".to_string(), &header).await?;
    client.request("test-addr", "测试!".to_string(), &header).await?;

    client
        .subscribe("test-topic", |(header, msg)| async move {
            println!("subscribe1");
            assert_eq!(header.get("k1").unwrap(), "v1");
            assert_eq!(msg, "测试!");
            Ok(())
        })
        .await?;

    client
        .subscribe("test-topic", |(header, msg)| async move {
            println!("subscribe2");
            assert_eq!(header.get("k1").unwrap(), "v1");
            assert_eq!(msg, "测试!");
            Ok(())
        })
        .await?;

    client.publish("test-topic", "测试!".to_string(), &header).await?;
    client.publish("test-topic", "测试!".to_string(), &header).await?;
    client.publish("test-topic", "测试!".to_string(), &header).await?;
    client.publish("test-topic", "测试!".to_string(), &header).await?;

    sleep(Duration::from_millis(1000)).await;

    Ok(())
}