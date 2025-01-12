use std::time::Duration;

use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::{testcontainers, tokio, TardisFuns};

use bios_iam::iam_constants;
use bios_iam::iam_test_helper::BIOSWebTestClient;

mod test_basic;
mod test_iam_scenes;

#[tokio::test]
async fn test_iam_api() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = test_basic::init(&docker).await?;

    let funs = iam_constants::get_tardis_inst();
    let (sysadmin_name, sysadmin_password) = bios_iam::iam_initializer::init_db(funs).await?.unwrap();

    tokio::spawn(async move {
        let web_server = TardisFuns::web_server();
        bios_iam::iam_initializer::init(web_server).await.unwrap();
        web_server.start().await.unwrap();
    });

    sleep(Duration::from_millis(500)).await;

    let mut client = BIOSWebTestClient::new("https://127.0.0.1:8080/iam".to_string());
    test_iam_scenes::test(&mut client, &sysadmin_name, &sysadmin_password).await?;

    Ok(())
}
