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
use bios::BIOSFuns;

use crate::domain::ident_domain::{IamAccount, IamAccountIdent, IamTenant, IamTenantCert, IamTenantIdent};
use crate::process::tenant_console::tc_tenant_dto::{
    TenantCertAddReq, TenantCertDetailResp, TenantCertModifyReq, TenantDetailResp, TenantIdentAddReq, TenantIdentDetailResp, TenantIdentModifyReq, TenantModifyReq,
};

#[put("/console/tenant/tenant")]
pub async fn modify_tenant(tenant_modify_req: Json<TenantModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;

    let mut values = Vec::new();
    if let Some(name) = &tenant_modify_req.name {
        values.push((IamTenant::Name, name.as_str().into()));
    }
    if let Some(icon) = &tenant_modify_req.icon {
        values.push((IamTenant::Icon, icon.as_str().into()));
    }
    if let Some(allow_account_register) = tenant_modify_req.allow_account_register {
        values.push((IamTenant::AllowAccountRegister, allow_account_register.into()));
    }
    if let Some(parameters) = &tenant_modify_req.parameters {
        values.push((IamTenant::Parameters, parameters.as_str().into()));
    }
    values.push((IamTenant::UpdateUser, ident_info.account_id.as_str().into()));

    BIOSFuns::reldb()
        .exec(
            &Query::update().table(IamTenant::Table).values(values).and_where(Expr::col(IamTenant::Id).eq(ident_info.tenant_id)).done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok("")
}

#[get("/console/tenant/tenant")]
pub async fn get_tenant(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamTenant::Table, IamTenant::Id),
            (IamTenant::Table, IamTenant::CreateTime),
            (IamTenant::Table, IamTenant::UpdateTime),
            (IamTenant::Table, IamTenant::Name),
            (IamTenant::Table, IamTenant::Icon),
            (IamTenant::Table, IamTenant::AllowAccountRegister),
            (IamTenant::Table, IamTenant::Parameters),
            (IamTenant::Table, IamTenant::Status),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamTenant::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamTenant::Table, IamTenant::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamTenant::Table, IamTenant::UpdateUser),
        )
        .and_where(Expr::tbl(IamTenant::Table, IamTenant::Id).eq(ident_info.tenant_id))
        .done();
    let item = BIOSFuns::reldb().fetch_one::<TenantDetailResp>(&sql_builder, None).await?;
    BIOSRespHelper::ok(item)
}

// ------------------------------------

#[post("/console/tenant/tenant/cert")]
pub async fn add_tenant_cert(tenant_cert_add_req: Json<TenantCertAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id = bios::basic::field::uuid();

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamTenantCert::Id])
                .from(IamTenantCert::Table)
                .and_where(Expr::col(IamTenantCert::Category).eq(tenant_cert_add_req.category.as_str()))
                .and_where(Expr::col(IamTenantCert::RelTenantId).eq(ident_info.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("TenantCert [category] already exists".to_string()));
    }

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamTenantCert::Table)
                .columns(vec![
                    IamTenantCert::Id,
                    IamTenantCert::CreateUser,
                    IamTenantCert::UpdateUser,
                    IamTenantCert::Category,
                    IamTenantCert::Version,
                    IamTenantCert::RelTenantId,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    ident_info.account_id.as_str().into(),
                    ident_info.account_id.as_str().into(),
                    tenant_cert_add_req.category.as_str().into(),
                    tenant_cert_add_req.version.into(),
                    ident_info.tenant_id.as_str().into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok(id)
}

#[put("/console/tenant/tenant/cert/{id}")]
pub async fn modify_tenant_cert(tenant_cert_modify_req: Json<TenantCertModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamTenantCert::Id])
                .from(IamTenantCert::Table)
                .and_where(Expr::col(IamTenantCert::Id).eq(id.as_str()))
                .and_where(Expr::col(IamTenantCert::RelTenantId).eq(ident_info.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("TenantCert not exists".to_string()));
    }

    let mut values = Vec::new();
    if let Some(version) = tenant_cert_modify_req.version {
        values.push((IamTenantCert::Version, version.into()));
    }
    values.push((IamTenantCert::UpdateUser, ident_info.account_id.as_str().into()));

    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamTenantCert::Table)
                .values(values)
                .and_where(Expr::col(IamTenantCert::Id).eq(id.as_str()))
                .and_where(Expr::col(IamTenantCert::RelTenantId).eq(ident_info.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok("")
}

#[get("/console/tenant/tenant/cert")]
pub async fn list_tenant_cert(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamTenantCert::Table, IamTenantCert::Id),
            (IamTenantCert::Table, IamTenantCert::CreateTime),
            (IamTenantCert::Table, IamTenantCert::UpdateTime),
            (IamTenantCert::Table, IamTenantCert::Category),
            (IamTenantCert::Table, IamTenantCert::Version),
            (IamTenantCert::Table, IamTenantCert::RelTenantId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamTenantCert::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamTenantCert::Table, IamTenantCert::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamTenantCert::Table, IamTenantCert::UpdateUser),
        )
        .and_where(Expr::tbl(IamTenantCert::Table, IamTenantCert::RelTenantId).eq(ident_info.tenant_id))
        .order_by(IamTenantCert::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().fetch_all::<TenantCertDetailResp>(&sql_builder, None).await?;
    BIOSRespHelper::ok(items)
}

#[delete("/console/tenant/tenant/cert/{id}")]
pub async fn delete_tenant_cert(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamTenantCert::Id])
                .from(IamTenantCert::Table)
                .and_where(Expr::col(IamTenantCert::Id).eq(id.as_str()))
                .and_where(Expr::col(IamTenantCert::RelTenantId).eq(ident_info.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("TenantCert not exists".to_string()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamTenantCert::iter().filter(|i| *i != IamTenantCert::Table))
        .from(IamTenantCert::Table)
        .and_where(Expr::col(IamTenantCert::Id).eq(id.as_str()))
        .and_where(Expr::col(IamTenantCert::RelTenantId).eq(ident_info.tenant_id))
        .done();
    BIOSFuns::reldb().soft_del(IamTenantCert::Table, IamTenantCert::Id, &ident_info.account_id, &sql_builder, &mut tx).await?;
    tx.commit().await?;
    BIOSRespHelper::ok("")
}

// ------------------------------------

#[post("/console/tenant/tenant/ident")]
pub async fn add_tenant_ident(tenant_ident_add_req: Json<TenantIdentAddReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id = bios::basic::field::uuid();

    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamTenantIdent::Id])
                .from(IamTenantIdent::Table)
                .and_where(Expr::col(IamTenantIdent::Kind).eq(tenant_ident_add_req.kind.to_string().to_lowercase()))
                .and_where(Expr::col(IamTenantIdent::RelTenantId).eq(ident_info.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("TenantIdent [kind] already exists".to_string()));
    }

    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(IamTenantIdent::Table)
                .columns(vec![
                    IamTenantIdent::Id,
                    IamTenantIdent::CreateUser,
                    IamTenantIdent::UpdateUser,
                    IamTenantIdent::Kind,
                    IamTenantIdent::ValidAkRuleNote,
                    IamTenantIdent::ValidAkRule,
                    IamTenantIdent::ValidSkRuleNote,
                    IamTenantIdent::ValidSkRule,
                    IamTenantIdent::ValidTime,
                    IamTenantIdent::RelTenantId,
                ])
                .values_panic(vec![
                    id.as_str().into(),
                    ident_info.account_id.as_str().into(),
                    ident_info.account_id.as_str().into(),
                    tenant_ident_add_req.kind.to_string().to_lowercase().into(),
                    tenant_ident_add_req.valid_ak_rule_note.as_deref().unwrap_or_default().into(),
                    tenant_ident_add_req.valid_ak_rule.as_deref().unwrap_or_default().into(),
                    tenant_ident_add_req.valid_sk_rule_note.as_deref().unwrap_or_default().into(),
                    tenant_ident_add_req.valid_sk_rule.as_deref().unwrap_or_default().into(),
                    tenant_ident_add_req.valid_time.into(),
                    ident_info.tenant_id.into(),
                ])
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok(id)
}

#[put("/console/tenant/tenant/ident/{id}")]
pub async fn modify_tenant_ident(tenant_ident_modify_req: Json<TenantIdentModifyReq>, req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamTenantIdent::Id])
                .from(IamTenantIdent::Table)
                .and_where(Expr::col(IamTenantIdent::Id).eq(id.as_str()))
                .and_where(Expr::col(IamTenantIdent::RelTenantId).eq(ident_info.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("TenantIdent not exists".to_string()));
    }

    let mut values = Vec::new();
    if let Some(valid_ak_rule_note) = &tenant_ident_modify_req.valid_ak_rule_note {
        values.push((IamTenantIdent::ValidAkRuleNote, valid_ak_rule_note.to_string().as_str().into()));
    }
    if let Some(valid_ak_rule) = &tenant_ident_modify_req.valid_ak_rule {
        values.push((IamTenantIdent::ValidAkRule, valid_ak_rule.to_string().as_str().into()));
    }
    if let Some(valid_sk_rule_note) = &tenant_ident_modify_req.valid_sk_rule_note {
        values.push((IamTenantIdent::ValidSkRuleNote, valid_sk_rule_note.to_string().as_str().into()));
    }
    if let Some(valid_sk_rule) = &tenant_ident_modify_req.valid_sk_rule {
        values.push((IamTenantIdent::ValidSkRule, valid_sk_rule.to_string().as_str().into()));
    }
    if let Some(valid_time) = tenant_ident_modify_req.valid_time {
        values.push((IamTenantIdent::ValidTime, valid_time.into()));
    }
    values.push((IamTenantIdent::UpdateUser, ident_info.account_id.as_str().into()));

    BIOSFuns::reldb()
        .exec(
            &Query::update()
                .table(IamTenantIdent::Table)
                .values(values)
                .and_where(Expr::col(IamTenantIdent::Id).eq(id.as_str()))
                .and_where(Expr::col(IamTenantIdent::RelTenantId).eq(ident_info.tenant_id))
                .done(),
            None,
        )
        .await?;
    BIOSRespHelper::ok("")
}

#[get("/console/tenant/tenant/ident")]
pub async fn list_tenant_ident(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;

    let create_user_table = Alias::new("create");
    let update_user_table = Alias::new("update");
    let sql_builder = Query::select()
        .columns(vec![
            (IamTenantIdent::Table, IamTenantIdent::Id),
            (IamTenantIdent::Table, IamTenantIdent::CreateTime),
            (IamTenantIdent::Table, IamTenantIdent::UpdateTime),
            (IamTenantIdent::Table, IamTenantIdent::Kind),
            (IamTenantIdent::Table, IamTenantIdent::ValidAkRuleNote),
            (IamTenantIdent::Table, IamTenantIdent::ValidAkRule),
            (IamTenantIdent::Table, IamTenantIdent::ValidSkRuleNote),
            (IamTenantIdent::Table, IamTenantIdent::ValidSkRule),
            (IamTenantIdent::Table, IamTenantIdent::ValidTime),
            (IamTenantIdent::Table, IamTenantIdent::RelTenantId),
        ])
        .expr_as(Expr::tbl(create_user_table.clone(), IamAccount::Name), Alias::new("create_user"))
        .expr_as(Expr::tbl(update_user_table.clone(), IamAccount::Name), Alias::new("update_user"))
        .from(IamTenantIdent::Table)
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            create_user_table.clone(),
            Expr::tbl(create_user_table, IamAccount::Id).equals(IamTenantIdent::Table, IamTenantIdent::CreateUser),
        )
        .join_as(
            JoinType::InnerJoin,
            IamAccount::Table,
            update_user_table.clone(),
            Expr::tbl(update_user_table, IamAccount::Id).equals(IamTenantIdent::Table, IamTenantIdent::UpdateUser),
        )
        .and_where(Expr::tbl(IamTenantIdent::Table, IamTenantIdent::RelTenantId).eq(ident_info.tenant_id))
        .order_by(IamTenantIdent::UpdateTime, Order::Desc)
        .done();
    let items = BIOSFuns::reldb().fetch_all::<TenantIdentDetailResp>(&sql_builder, None).await?;
    BIOSRespHelper::ok(items)
}

#[delete("/console/tenant/tenant/ident/{id}")]
pub async fn delete_tenant_ident(req: HttpRequest) -> BIOSResp {
    let ident_info = get_ident_account_info(&req)?;
    let id: String = req.match_info().get("id").unwrap().parse()?;

    if !BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![IamTenantIdent::Id])
                .from(IamTenantIdent::Table)
                .and_where(Expr::col(IamTenantIdent::Id).eq(id.as_str()))
                .and_where(Expr::col(IamTenantIdent::RelTenantId).eq(ident_info.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::NotFound("TenantIdent not exists".to_string()));
    }
    if BIOSFuns::reldb()
        .exists(
            &Query::select()
                .columns(vec![(IamAccountIdent::Table, IamAccountIdent::Id)])
                .from(IamAccountIdent::Table)
                .inner_join(
                    IamTenantIdent::Table,
                    Expr::tbl(IamTenantIdent::Table, IamTenantIdent::Kind).equals(IamAccountIdent::Table, IamAccountIdent::Kind),
                )
                .and_where(Expr::tbl(IamTenantIdent::Table, IamTenantIdent::Id).eq(id.as_str()))
                .and_where(Expr::tbl(IamTenantIdent::Table, IamTenantIdent::RelTenantId).eq(ident_info.tenant_id.as_str()))
                .done(),
            None,
        )
        .await?
    {
        return BIOSRespHelper::bus_error(BIOSError::Conflict("Please delete the associated [account_ident] data first".to_owned()));
    }

    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    let sql_builder = Query::select()
        .columns(IamTenantIdent::iter().filter(|i| *i != IamTenantIdent::Table))
        .from(IamTenantIdent::Table)
        .and_where(Expr::col(IamTenantIdent::Id).eq(id.as_str()))
        .and_where(Expr::col(IamTenantIdent::RelTenantId).eq(ident_info.tenant_id))
        .done();
    BIOSFuns::reldb().soft_del(IamTenantIdent::Table, IamTenantIdent::Id, &ident_info.account_id, &sql_builder, &mut tx).await?;
    tx.commit().await?;
    BIOSRespHelper::ok("")
}