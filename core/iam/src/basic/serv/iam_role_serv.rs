use async_trait::async_trait;
use sea_orm::EntityName;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::SelectStatement;
use tardis::web::web_resp::TardisPage;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemModifyReq};
use bios_basic::rbum::dto::rbum_rel_dto::{RbumRelBoneResp, RbumRelCheckReq};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::helper::rbum_scope_helper::get_scope_level_by_context;
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;

use crate::basic::domain::iam_role;
use crate::basic::dto::iam_filer_dto::IamRoleFilterReq;
use crate::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq, IamRoleAggModifyReq, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::iam_config::{IamBasicConfigApi, IamBasicInfoManager, IamConfig};
use crate::iam_constants;
use crate::iam_constants::{RBUM_SCOPE_LEVEL_APP, RBUM_SCOPE_LEVEL_TENANT};
use crate::iam_enumeration::IamRelKind;

pub struct IamRoleServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_role::ActiveModel, IamRoleAddReq, IamRoleModifyReq, IamRoleSummaryResp, IamRoleDetailResp, IamRoleFilterReq> for IamRoleServ {
    fn get_ext_table_name() -> &'static str {
        iam_role::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        IamBasicInfoManager::get_config(|conf| conf.kind_role_id.clone())
    }

    fn get_rbum_domain_id() -> String {
        IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone())
    }

    async fn package_item_add(add_req: &IamRoleAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: None,
            code: Some(add_req.code.clone()),
            name: add_req.name.clone(),
            disabled: None,
            scope_level: add_req.scope_level.clone(),
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamRoleAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<iam_role::ActiveModel> {
        Ok(iam_role::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            ..Default::default()
        })
    }

    async fn after_add_item(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let role = Self::do_get_item(
            id,
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            cxt,
        )
        .await?;
        funs.cache()
            .set(
                &format!("{}{}", funs.conf::<IamConfig>().cache_key_role_info_, id),
                TardisFuns::json.obj_to_string(&role)?.as_str(),
            )
            .await?;
        Ok(())
    }

    async fn package_item_modify(_: &str, modify_req: &IamRoleModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
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

    async fn package_ext_modify(id: &str, modify_req: &IamRoleModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<iam_role::ActiveModel>> {
        if modify_req.icon.is_none() && modify_req.sort.is_none() {
            return Ok(None);
        }
        let mut iam_role = iam_role::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            iam_role.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            iam_role.sort = Set(sort);
        }
        Ok(Some(iam_role))
    }

    async fn after_modify_item(id: &str, modify_req: &mut IamRoleModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let role = Self::do_get_item(
            id,
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            cxt,
        )
        .await?;
        funs.cache()
            .set(
                &format!("{}{}", funs.conf::<IamConfig>().cache_key_role_info_, id),
                TardisFuns::json.obj_to_string(&role)?.as_str(),
            )
            .await?;
        let role_id = id.to_string();
        if modify_req.disabled.unwrap_or(false) {
            let cxt = cxt.clone();
            tardis::tokio::spawn(async move {
                let funs = iam_constants::get_tardis_inst();
                let mut count = IamRoleServ::count_rel_accounts(&role_id, &funs, &cxt).await.unwrap() as isize;
                let mut page_number = 1;
                while count > 0 {
                    let ids = IamRoleServ::paginate_id_rel_accounts(&role_id, page_number, 100, None, None, &funs, &cxt).await.unwrap().records;
                    for id in ids {
                        IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(&id, &funs).await.unwrap();
                    }
                    page_number += 1;
                    count -= 100;
                }
            });
        }
        Ok(())
    }

    async fn after_delete_item(id: &str, _: Option<IamRoleDetailResp>, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        funs.cache().del(&format!("{}{}", funs.conf::<IamConfig>().cache_key_role_info_, id)).await?;
        let role_id = id.to_string();
        let cxt = cxt.clone();
        tardis::tokio::spawn(async move {
            let funs = iam_constants::get_tardis_inst();
            let mut count = IamRoleServ::count_rel_accounts(&role_id, &funs, &cxt).await.unwrap() as isize;
            let mut page_number = 1;
            while count > 0 {
                let ids = IamRoleServ::paginate_id_rel_accounts(&role_id, page_number, 100, None, None, &funs, &cxt).await.unwrap().records;
                for id in ids {
                    IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(&id, &funs).await.unwrap();
                }
                page_number += 1;
                count -= 100;
            }
        });
        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, _: &IamRoleFilterReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_role::Entity, iam_role::Column::Icon));
        query.column((iam_role::Entity, iam_role::Column::Sort));
        Ok(())
    }

    async fn get_item(id: &str, filter: &IamRoleFilterReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamRoleDetailResp> {
        if let Some(role) = funs.cache().get(&format!("{}{}", funs.conf::<IamConfig>().cache_key_role_info_, id)).await? {
            let role = TardisFuns::json.str_to_obj::<IamRoleDetailResp>(&role)?;
            if rbum_scope_helper::check_scope(&role.own_paths, Some(role.scope_level.to_int()), &filter.basic, &cxt) {
                return Ok(role);
            }
        }
        let role = Self::do_get_item(id, filter, funs, cxt).await?;
        funs.cache()
            .set(
                &format!("{}{}", funs.conf::<IamConfig>().cache_key_role_info_, id),
                TardisFuns::json.obj_to_string(&role)?.as_str(),
            )
            .await?;
        Ok(role)
    }
}

impl<'a> IamRoleServ {
    pub async fn add_role(add_req: &mut IamRoleAggAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let role_id = Self::add_item(&mut add_req.role, funs, cxt).await?;
        if let Some(res_ids) = &add_req.res_ids {
            for res_id in res_ids {
                Self::add_rel_res(&role_id, res_id, funs, cxt).await?;
            }
        }
        Ok(role_id)
    }

    pub async fn modify_role(id: &str, modify_req: &mut IamRoleAggModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::modify_item(id, &mut modify_req.role, funs, cxt).await?;
        if let Some(input_res_ids) = &modify_req.res_ids {
            let stored_res = Self::find_simple_rel_res(id, None, None, funs, cxt).await?;
            let stored_res_ids: Vec<String> = stored_res.into_iter().map(|x| x.rel_id).collect();
            for input_res_id in input_res_ids {
                if !stored_res_ids.contains(input_res_id) {
                    Self::add_rel_res(id, input_res_id, funs, cxt).await?;
                }
            }
            for stored_res_id in stored_res_ids {
                if !input_res_ids.contains(&stored_res_id) {
                    Self::delete_rel_res(id, &stored_res_id, funs, cxt).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn add_rel_account(role_id: &str, account_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let scope_level = get_scope_level_by_context(cxt)?;
        if scope_level == RBUM_SCOPE_LEVEL_APP && (role_id == funs.iam_basic_role_sys_admin_id() || role_id == funs.iam_basic_role_tenant_admin_id())
            || scope_level == RBUM_SCOPE_LEVEL_TENANT && role_id == funs.iam_basic_role_sys_admin_id()
        {
            return Err(TardisError::BadRequest("The associated role is invalid.".to_string()));
        }
        // TODO only bind the same own_paths roles
        // E.g. sys admin can't bind tenant admin
        IamRelServ::add_simple_rel(&IamRelKind::IamAccountRole, account_id, role_id, None, None, funs, cxt).await
    }

    pub async fn delete_rel_account(role_id: &str, account_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamAccountRole, account_id, role_id, funs, cxt).await
    }

    pub async fn count_rel_accounts(role_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamRelServ::count_to_rels(&IamRelKind::IamAccountRole, role_id, funs, cxt).await
    }

    pub async fn find_id_rel_accounts(
        role_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        IamRelServ::find_to_id_rels(&IamRelKind::IamAccountRole, role_id, desc_by_create, desc_by_update, funs, cxt).await
    }

    pub async fn find_simple_rel_accounts(
        role_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_to_simple_rels(&IamRelKind::IamAccountRole, role_id, desc_by_create, desc_by_update, funs, cxt).await
    }

    pub async fn paginate_id_rel_accounts(
        role_id: &str,
        page_number: u64,
        page_size: u64,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        IamRelServ::paginate_to_id_rels(&IamRelKind::IamAccountRole, role_id, page_number, page_size, desc_by_create, desc_by_update, funs, cxt).await
    }

    pub async fn paginate_simple_rel_accounts(
        role_id: &str,
        page_number: u64,
        page_size: u64,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        IamRelServ::paginate_to_simple_rels(&IamRelKind::IamAccountRole, role_id, page_number, page_size, desc_by_create, desc_by_update, funs, cxt).await
    }

    pub async fn add_rel_res(role_id: &str, res_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRelServ::add_simple_rel(&IamRelKind::IamResRole, res_id, role_id, None, None, funs, cxt).await
    }

    pub async fn delete_rel_res(role_id: &str, res_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamResRole, res_id, role_id, funs, cxt).await
    }

    pub async fn count_rel_res(role_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamRelServ::count_to_rels(&IamRelKind::IamResRole, role_id, funs, cxt).await
    }

    pub async fn find_id_rel_res(
        role_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        IamRelServ::find_to_id_rels(&IamRelKind::IamResRole, role_id, desc_by_create, desc_by_update, funs, cxt).await
    }

    pub async fn find_simple_rel_res(
        role_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_to_simple_rels(&IamRelKind::IamResRole, role_id, desc_by_create, desc_by_update, funs, cxt).await
    }

    pub async fn paginate_id_rel_res(
        role_id: &str,
        page_number: u64,
        page_size: u64,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        IamRelServ::paginate_to_id_rels(&IamRelKind::IamResRole, role_id, page_number, page_size, desc_by_create, desc_by_update, funs, cxt).await
    }

    pub async fn paginate_simple_rel_res(
        role_id: &str,
        page_number: u64,
        page_size: u64,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        IamRelServ::paginate_to_simple_rels(&IamRelKind::IamResRole, role_id, page_number, page_size, desc_by_create, desc_by_update, funs, cxt).await
    }

    pub async fn need_sys_admin(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::need_role(&funs.iam_basic_role_sys_admin_id(), funs, cxt).await
    }

    pub async fn need_tenant_admin(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::need_role(&funs.iam_basic_role_tenant_admin_id(), funs, cxt).await
    }

    pub async fn need_app_admin(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::need_role(&funs.iam_basic_role_app_admin_id(), funs, cxt).await
    }

    pub async fn need_role(role_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let exist = RbumRelServ::check_rel(
            &mut RbumRelCheckReq {
                tag: IamRelKind::IamAccountRole.to_string(),
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: cxt.owner.clone(),
                to_rbum_item_id: role_id.to_string(),
                from_attrs: Default::default(),
                to_attrs: Default::default(),
            },
            funs,
            cxt,
        )
        .await?;
        if !exist {
            Err(TardisError::Unauthorized("illegal operation".to_string()))
        } else {
            Ok(())
        }
    }
}
