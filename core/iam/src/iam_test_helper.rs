use poem_openapi::types::{ParseFromJSON, ToJSON};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::log::{info, warn};
use tardis::web::web_client::TardisWebClient;
use tardis::web::web_resp::{TardisResp, Void};
use tardis::TardisFuns;

use crate::basic::dto::iam_cert_dto::IamContextFetchReq;

pub struct BIOSWebTestClient {
    client: TardisWebClient,
    context: TardisContext,
    base_url: String,
}

impl BIOSWebTestClient {
    pub fn new(base_url: String) -> BIOSWebTestClient {
        BIOSWebTestClient {
            client: TardisWebClient::init(600).unwrap(),
            context: Default::default(),
            base_url,
        }
    }

    pub async fn set_auth(&mut self, token: &str, app_id: Option<String>) -> TardisResult<()> {
        self.context = self.put("/cp/context", &IamContextFetchReq { token: token.to_string(), app_id }).await;
        self.set_default_header(
            &TardisFuns::fw_config().web_server.context_conf.context_header_name,
            TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&self.context)?).as_str(),
        );
        Ok(())
    }

    pub fn context(&self) -> &TardisContext {
        &self.context
    }

    pub fn set_default_header(&mut self, key: &str, value: &str) {
        self.client.remove_default_header(key);
        self.client.set_default_header(key, value);
    }

    pub async fn get_to_str(&self, url: &str) -> String {
        self.client.get_to_str(format!("{}{}", self.base_url, url).as_str(), None).await.unwrap().body.unwrap()
    }

    pub async fn get<T>(&self, url: &str) -> T
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync + Debug,
    {
        let result: TardisResp<T> = self.client.get::<TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("#####[GET]|{}:{:#?}", url, result);
        result.data.unwrap()
    }

    pub async fn delete(&self, url: &str) {
        let result: TardisResp<Void> = self.client.delete(format!("{}{}", self.base_url, url).as_str(), None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("#####[DELETE]|{}:{:#?}", url, result);
    }

    pub async fn post<B: Serialize, T>(&self, url: &str, body: &B) -> T
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync + Debug,
    {
        let result: TardisResp<T> = self.client.post::<B, TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), body, None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("#####[POST]|{}:{:#?}", url, result);
        result.data.unwrap()
    }

    pub async fn put<B: Serialize, T>(&self, url: &str, body: &B) -> T
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync + Debug,
    {
        let result: TardisResp<T> = self.client.put::<B, TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), body, None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("#####[PUT]|{}:{:#?}", url, result);
        result.data.unwrap()
    }

    pub async fn patch<B: Serialize, T>(&self, url: &str, body: &B) -> T
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync + Debug,
    {
        let result: TardisResp<T> = self.client.patch::<B, TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), body, None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("#####[PATCH]|{}:{:#?}", url, result);
        result.data.unwrap()
    }
}
