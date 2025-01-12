use async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{Expr, SelectStatement};
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_filer_dto::RbumItemRelFilterReq;
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemModifyReq};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_app;
use crate::basic::dto::iam_app_dto::{IamAppAddReq, IamAppDetailResp, IamAppModifyReq, IamAppSummaryResp};
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::iam_config::IamBasicInfoManager;
use crate::iam_constants::{RBUM_ITEM_ID_APP_LEN, RBUM_SCOPE_LEVEL_APP};
use crate::iam_enumeration::IamRelKind;

pub struct IamAppServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_app::ActiveModel, IamAppAddReq, IamAppModifyReq, IamAppSummaryResp, IamAppDetailResp, IamAppFilterReq> for IamAppServ {
    fn get_ext_table_name() -> &'static str {
        iam_app::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        IamBasicInfoManager::get_config(|conf| conf.kind_app_id.clone())
    }

    fn get_rbum_domain_id() -> String {
        IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone())
    }

    async fn package_item_add(add_req: &IamAppAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: add_req.id.clone(),
            code: None,
            name: add_req.name.clone(),
            disabled: add_req.disabled,
            scope_level: add_req.scope_level.clone(),
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamAppAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<iam_app::ActiveModel> {
        Ok(iam_app::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            contact_phone: Set(add_req.contact_phone.as_ref().unwrap_or(&"".to_string()).to_string()),
            ..Default::default()
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamAppModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
        if modify_req.name.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemModifyReq {
            code: None,
            name: modify_req.name.clone(),
            scope_level: modify_req.scope_level.clone(),
            disabled: modify_req.disabled,
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &IamAppModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<iam_app::ActiveModel>> {
        if modify_req.icon.is_none() && modify_req.sort.is_none() && modify_req.contact_phone.is_none() {
            return Ok(None);
        }
        let mut iam_app = iam_app::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            iam_app.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            iam_app.sort = Set(sort);
        }
        if let Some(contact_phone) = &modify_req.contact_phone {
            iam_app.contact_phone = Set(contact_phone.to_string());
        }
        Ok(Some(iam_app))
    }

    async fn after_modify_item(id: &str, modify_req: &mut IamAppModifyReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
        if modify_req.disabled.unwrap_or(false) {
            IamIdentCacheServ::delete_tokens_and_contexts_by_tenant_or_app(id, true, funs, ctx).await?;
        }
        Ok(())
    }

    async fn before_delete_item(_: &str, funs: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<IamAppDetailResp>> {
        Err(funs.err().conflict(&Self::get_obj_name(), "delete", "app can only be disabled but not deleted"))
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamAppFilterReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_app::Entity, iam_app::Column::ContactPhone));
        query.column((iam_app::Entity, iam_app::Column::Icon));
        query.column((iam_app::Entity, iam_app::Column::Sort));
        if let Some(contact_phone) = &filter.contact_phone {
            query.and_where(Expr::col(iam_app::Column::ContactPhone).eq(contact_phone.as_str()));
        }
        Ok(())
    }
}

impl<'a> IamAppServ {
    pub fn get_new_id() -> String {
        TardisFuns::field.nanoid_len(RBUM_ITEM_ID_APP_LEN as usize)
    }

    pub fn get_id_by_ctx(ctx: &TardisContext, funs: &TardisFunsInst<'a>) -> TardisResult<String> {
        if let Some(id) = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_APP.to_int(), &ctx.own_paths) {
            Ok(id)
        } else {
            Err(funs.err().unauthorized(&Self::get_obj_name(), "get_id", &format!("app id not found in tardis content {}", ctx.own_paths)))
        }
    }

    pub async fn add_rel_account(app_id: &str, account_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::add_simple_rel(&IamRelKind::IamAccountApp, account_id, app_id, None, None, funs, ctx).await
    }

    pub async fn delete_rel_account(app_id: &str, account_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamAccountApp, account_id, app_id, funs, ctx).await
    }

    pub async fn count_rel_accounts(app_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<u64> {
        IamRelServ::count_to_rels(&IamRelKind::IamAccountApp, app_id, funs, ctx).await
    }

    pub async fn exist_rel_accounts(app_id: &str, account_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<bool> {
        IamRelServ::exist_rels(&IamRelKind::IamAccountApp, account_id, app_id, funs, ctx).await
    }

    pub fn with_app_rel_filter(ctx: &TardisContext, funs: &TardisFunsInst<'a>) -> TardisResult<Option<RbumItemRelFilterReq>> {
        Ok(Some(RbumItemRelFilterReq {
            rel_by_from: true,
            tag: Some(IamRelKind::IamAccountApp.to_string()),
            from_rbum_kind: Some(RbumRelFromKind::Item),
            rel_item_id: Some(Self::get_id_by_ctx(ctx, funs)?),
        }))
    }
}
