/*
 * Copyright 2021. gudaoxuri
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

use actix_web::body::AnyBody;
use actix_web::http::StatusCode;
use actix_web::test::{call_service, read_body_json};
use actix_web::{test, App};
use chrono::Utc;
use testcontainers::clients;

use bios::basic::config::FrameworkConfig;
use bios::basic::error::BIOSResult;
use bios::db::reldb_client::BIOSPage;
use bios::web::resp_handler::BIOSRespHelper;
use bios::web::web_server::BIOSWebServer;
use bios::BIOSFuns;
use bios_baas_iam::iam_config::WorkSpaceConfig;
use bios_baas_iam::process::app_console;
use bios_baas_iam::process::app_console::ac_auth_policy_dto::{AuthPolicyAddReq, AuthPolicyDetailResp, AuthPolicyModifyReq, AuthPolicySubjectAddReq, AuthPolicySubjectDetailResp};
use bios_baas_iam::process::app_console::ac_resource_dto::{ResourceAddReq, ResourceSubjectAddReq};
use bios_baas_iam::process::basic_dto::{AuthResultKind, AuthSubjectKind, AuthSubjectOperatorKind, OptActionKind, ResourceKind};

#[actix_rt::test]
async fn test_auth_policy() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = crate::test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(app_console::ac_resource_processor::add_resource_subject)
            .service(app_console::ac_resource_processor::add_resource)
            .service(app_console::ac_auth_policy_processor::add_auth_policy)
            .service(app_console::ac_auth_policy_processor::modify_auth_policy)
            .service(app_console::ac_auth_policy_processor::list_auth_policy)
            .service(app_console::ac_auth_policy_processor::delete_auth_policy),
    )
    .await;

    // Add AuthPolicy
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/auth-policy")
        .set_json(&AuthPolicyAddReq {
            name: "测试策略".to_string(),
            valid_start_time: 0,
            valid_end_time: 0,
            rel_resource_id: "ddddd".to_string(),
            action_kind: OptActionKind::Fetch,
            result_kind: AuthResultKind::Accept,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "404");

    // Add ResourceSubject
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/resource/subject")
        .set_json(&ResourceSubjectAddReq {
            code_postfix: "httpbin".to_string(),
            name: "测试Http请求".to_string(),
            sort: 0,
            kind: ResourceKind::Api,
            uri: "http://httpbin.org".to_string(),
            ak: None,
            sk: None,
            platform_account: None,
            platform_project_id: None,
            timeout_ms: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let subject_id = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await.body.unwrap();

    // Add Resource
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/resource")
        .set_json(&ResourceAddReq {
            name: "测试Get请求".to_string(),
            path_and_query: "/get".to_string(),
            icon: "xxx.png".to_string(),
            action: None,
            sort: 1,
            res_group: false,
            parent_id: None,
            rel_resource_subject_id: subject_id.clone(),
            expose_kind: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let resource_id = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await.body.unwrap();

    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/auth-policy")
        .set_json(&AuthPolicyAddReq {
            name: "测试策略".to_string(),
            valid_start_time: Utc::now().timestamp(),
            valid_end_time: Utc::now().timestamp() + 3600,
            rel_resource_id: resource_id.clone(),
            action_kind: OptActionKind::Fetch,
            result_kind: AuthResultKind::Accept,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let id = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await.body.unwrap();

    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/auth-policy")
        .set_json(&AuthPolicyAddReq {
            name: "测试策略".to_string(),
            valid_start_time: Utc::now().timestamp() + 100,
            valid_end_time: Utc::now().timestamp() + 1000,
            rel_resource_id: resource_id.clone(),
            action_kind: OptActionKind::Fetch,
            result_kind: AuthResultKind::Accept,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "409");

    // Modify AuthPolicy
    let req = test::TestRequest::put()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/app/auth-policy/{}", id.clone()).as_str())
        .set_json(&AuthPolicyModifyReq {
            name: None,
            valid_start_time: Some(Utc::now().timestamp() - 1000),
            valid_end_time: Some(Utc::now().timestamp() + 36000),
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    // List AuthPolicy
    let req = test::TestRequest::get()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/auth-policy?page_number=1&page_size=10")
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSRespHelper<BIOSPage<AuthPolicyDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body.total_size, 1);
    assert_eq!(body.records[0].name, "测试策略");
    assert_eq!(body.records[0].create_user, "平台管理员");
    assert_eq!(body.records[0].update_user, "平台管理员");

    // Delete AuthPolicy
    let req = test::TestRequest::delete()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/app/auth-policy/{}", id.clone()).as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}

#[actix_rt::test]
async fn test_auth_policy_subject() -> BIOSResult<()> {
    let docker = clients::Cli::default();
    let _c = crate::test_basic::init(&docker).await;
    let app = test::init_service(
        App::new()
            .wrap(BIOSWebServer::init_cors(&FrameworkConfig::default()))
            .wrap(BIOSWebServer::init_error_handlers())
            .service(app_console::ac_resource_processor::add_resource_subject)
            .service(app_console::ac_resource_processor::add_resource)
            .service(app_console::ac_auth_policy_processor::add_auth_policy)
            .service(app_console::ac_auth_policy_processor::add_auth_policy_subject)
            .service(app_console::ac_auth_policy_processor::list_auth_policy_subject)
            .service(app_console::ac_auth_policy_processor::delete_auth_policy_subject),
    )
    .await;

    // Add ResourceSubject
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/resource/subject")
        .set_json(&ResourceSubjectAddReq {
            code_postfix: "httpbin".to_string(),
            name: "测试Http请求".to_string(),
            sort: 0,
            kind: ResourceKind::Api,
            uri: "http://httpbin.org".to_string(),
            ak: None,
            sk: None,
            platform_account: None,
            platform_project_id: None,
            timeout_ms: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let subject_id = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await.body.unwrap();

    // Add Resource
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/resource")
        .set_json(&ResourceAddReq {
            name: "测试Get请求".to_string(),
            path_and_query: "/get".to_string(),
            icon: "xxx.png".to_string(),
            action: None,
            sort: 1,
            res_group: false,
            parent_id: None,
            rel_resource_subject_id: subject_id.clone(),
            expose_kind: None,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let resource_id = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await.body.unwrap();

    // Add AuthPolicy
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri("/console/app/auth-policy")
        .set_json(&AuthPolicyAddReq {
            name: "测试策略".to_string(),
            valid_start_time: Utc::now().timestamp(),
            valid_end_time: Utc::now().timestamp() + 3600,
            rel_resource_id: resource_id.clone(),
            action_kind: OptActionKind::Fetch,
            result_kind: AuthResultKind::Accept,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let auth_policy_id = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await.body.unwrap();

    // Add AuthPolicySubject
    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/app/auth-policy/{}/subject", auth_policy_id.clone()).as_str())
        .set_json(&AuthPolicySubjectAddReq {
            subject_kind: AuthSubjectKind::Tenant,
            subject_id: "t001".to_string(),
            subject_operator: AuthSubjectOperatorKind::Eq,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "404");

    let req = test::TestRequest::post()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/app/auth-policy/{}/subject", auth_policy_id.clone()).as_str())
        .set_json(&AuthPolicySubjectAddReq {
            subject_kind: AuthSubjectKind::Account,
            subject_id: "admin001".to_string(),
            subject_operator: AuthSubjectOperatorKind::Eq,
        })
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let id = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await.body.unwrap();

    // List AuthPolicySubject
    let req = test::TestRequest::get()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/app/auth-policy/{}/subject", auth_policy_id.clone()).as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_json::<BIOSRespHelper<Vec<AuthPolicySubjectDetailResp>>, AnyBody>(resp).await.body.unwrap();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0].subject_kind, "account");
    assert_eq!(body[0].subject_id, "admin001");
    assert_eq!(body[0].create_user, "平台管理员");
    assert_eq!(body[0].update_user, "平台管理员");

    let cache = BIOSFuns::cache()
        .hget(&BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_resources, "fetch##http://httpbin.org/get")
        .await?
        .unwrap();
    assert_eq!(bios::basic::json::str_to_json(&cache).unwrap()["account"], "#admin001#");

    // Delete AuthPolicySubject
    let req = test::TestRequest::delete()
        .insert_header((
            BIOSFuns::fw_config().web.ident_info_flag.clone(),
            bios::basic::security::digest::base64::encode(r#"{"app_id":"app1","tenant_id":"tenant1","account_id":"admin001","ak":"ak1","token":"t01"}"#),
        ))
        .uri(format!("/console/app/auth-policy/{}/subject/{}", auth_policy_id.clone().as_str(), id.clone()).as_str())
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = read_body_json::<BIOSRespHelper<String>, AnyBody>(resp).await;
    assert_eq!(result.code, "200");

    Ok(())
}