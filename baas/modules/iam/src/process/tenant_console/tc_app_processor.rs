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

use actix_web::{delete, get, post, put, HttpRequest};
use sea_query::{Alias, Expr, JoinType, Order, Query};
use sqlx::Connection;
use strum::IntoEnumIterator;

use bios::basic::error::BIOSError;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::basic_processor::get_ident_account_info;
use bios::web::resp_handler::{BIOSResp, BIOSRespHelper};
use bios::web::validate::json::Json;
use bios::web::validate::query::Query as VQuery;
use bios::BIOSFuns;

use crate::domain::ident_domain::{IamAccount, IamApp, IamAppIdent};
use crate::iam_config::WorkSpaceConfig;
use crate::process::basic_dto::CommonStatus;
use crate::process::tenant_console::tc_app_dto::{AppAddReq, AppDetailResp, AppModifyReq, AppQueryReq};
use chrono::Utc;

#[post("/console/tenant/app")]
pub async fn add_app(app_add_req: Json<AppAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id = bios::basic::field::uuid();

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamApp::Table)
                .columns(vec![
                    IamApp::Id,
                    IamApp::CreateUser,
                    IamApp::UpdateUser,
                    IamApp::Name,
                    IamApp::Icon,
                    IamApp::Parameters,
                    IamApp::Status,
                    IamApp::RelTenantId,
                ])
                .values_panic(vec![
                    id.clone().into(),
                    ident_info.account_id.clone().into(),
                    ident_info.account_id.clone().into(),
                    app_add_req.name.clone().into(),
                    app_add_req.icon.clone().unwrap_or_default().into(),
                    app_add_req.parameters.clone().unwrap_or_default().into(),
                    CommonStatus::Enabled.to_string().to_lowercase().into(),
                    ident_info.tenant_id.into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok(id)
}

#[put("/console/tenant/app/{id}")]
pub async fn modify_app(app_modify_req: Json<AppModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamApp::Id])
                .from(IamApp::Table)
                .and_where(Expr::col(IamApp::Id).eq(id.clone()))
                .and_where(Expr::col(IamApp::RelTenantId).eq(ident_info.tenant_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("App not exists".to_string()));
    }

    let mut values = Vec::new();
    if let Some(name) = &app_modify_req.name {
        values.push((IamApp::Name, name.to_string().into()));
    }
    if let Some(parameters) = &app_modify_req.parameters {
        values.push((IamApp::Parameters, parameters.to_string().into()));
    }
    if let Some(icon) = &app_modify_req.icon {
        values.push((IamApp::Icon, icon.to_string().into()));
    }
    if let Some(status) = &app_modify_req.status {
        values.push((IamApp::Status, status.to_string().to_lowercase().into()));
    }
    values.push((IamApp::UpdateUser, ident_info.account_id.clone().into()));

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamApp::Table)
                .values(values)
                .and_where(Expr::col(IamApp::Id).eq(id.clone()))
                .and_where(Expr::col(IamApp::RelTenantId).eq(ident_info.tenant_id.clone()))
                .done(),
            Some(&mut tx),
        )
        .await?;
    if let Some(status) = &app_modify_req.status {
        let aksks = BIOSFuns::reldb()
            .fetch_all::<AkSkInfoResp>(
                &Query::select()
                    .columns(vec![IamAppIdent::Ak, IamAppIdent::Sk, IamAppIdent::ValidTime])
                    .from(IamAppIdent::Table)
                    .and_where(Expr::col(IamAppIdent::RelAppId).eq(id.clone()))
                    .done(),
                None,
            )
            .await?;
        match status {
            CommonStatus::Enabled => {
                for aksk_resp in aksks {
                    BIOSFuns::cache()
                        .set_ex(
                            format!("{}{}", &BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_aksk, aksk_resp.ak).as_str(),
                            format!("{}:{}:{}", aksk_resp.sk, ident_info.tenant_id.clone(), id.clone()).as_str(),
                            (aksk_resp.valid_time - Utc::now().timestamp()) as usize,
                        )
                        .await?;
                }
            }
            CommonStatus::Disabled => {
                for aksk_resp in aksks {
                    BIOSFuns::cache().del(format!("{}{}", &BIOSFuns::ws_config::<WorkSpaceConfig>().iam.cache_aksk, aksk_resp.ak).as_str()).await?;
                }
            }
        }
    }
    tx.commit().await?;

    BIOSRespHelper::ok("")
}

#[get("/console/tenant/app")]
pub async fn list_app(query: VQuery<AppQueryReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamApp::Table, IamApp::Id),
            (IamApp::Table, IamApp::CreateTime),
            (IamApp::Table, IamApp::UpdateTime),
            (IamApp::Table, IamApp::Name),
            (IamApp::Table, IamApp::Icon),
            (IamApp::Table, IamApp::Parameters),
            (IamApp::Table, IamApp::Status),
            (IamApp::Table, IamApp::RelTenantId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamApp::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamApp::Table, IamApp::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamApp::Table, IamApp::UpdateUser),
        )
        .and_where_option(if let Some(name) = &query.name {
            Some(Expr::tbl(IamApp::Table, IamApp::Name).like(format!("%{}%", name).as_str()))
        } else {
            None
        })
        .and_where(Expr::tbl(IamApp::Table, IamApp::RelTenantId).eq(ident_info.tenant_id))
        .order_by(IamApp::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().pagination::<AppDetailResp>(&sql_builder, query.page_number, query.page_size, None).await?;
    BIOSRespHelper::ok(items)
}

#[delete("/console/tenant/app/{id}")]
pub async fn delete_app(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamApp::Id])
                .from(IamApp::Table)
                .and_where(Expr::col(IamApp::Id).eq(id.clone()))
                .and_where(Expr::col(IamApp::RelTenantId).eq(ident_info.tenant_id.clone()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("App not exists".to_string()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    // TODO 级联删除 IamAppIdent IamAccountApp IamGroup IamGroupNode  IamAccountGroup  IamRole  IamAccountRole  IamResourceSubject  IamResource  IamAuthPolicy  IamAuthPolicySubject

    let sql_builder = Query::select().columns(IamApp::iter().filter(|i| *i != IamApp::Table)).from(IamApp::Table).and_where(Expr::col(IamApp::Id).eq(id.clone())).done();
    BIOSFuns::reldb().soft_del(IamApp::Table, IamApp::Id, &ident_info.account_id, &sql_builder, &mut tx).await?;

    tx.commit().await?;
    BIOSRespHelper::ok("")
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct AkSkInfoResp {
    pub ak: String,
    pub sk: String,
    pub valid_time: i64,
}
