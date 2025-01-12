use std::collections::HashMap;

use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::chrono::{Duration, Utc};
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq};
use bios_basic::rbum::dto::rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelEnvAggAddReq};
use bios_basic::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelBoneResp, RbumRelFindReq};
use bios_basic::rbum::rbum_enumeration::{RbumRelEnvKind, RbumRelFromKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;

use crate::basic::dto::iam_filer_dto::IamResFilterReq;
use crate::basic::serv::iam_key_cache_serv::{IamCacheResRelAddOrModifyReq, IamCacheResRelDeleteReq, IamIdentCacheServ, IamResCacheServ};
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::iam_enumeration::{IamRelKind, IamResKind};

pub struct IamRelServ;

impl<'a> IamRelServ {
    pub async fn add_simple_rel(
        rel_kind: &IamRelKind,
        from_iam_item_id: &str,
        to_iam_item_id: &str,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let value1 = start_timestamp.unwrap_or_else(|| Utc::now().timestamp());
        let value2 = end_timestamp.unwrap_or_else(|| (Utc::now() + Duration::days(365 * 100)).timestamp());
        let req = &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: rel_kind.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: from_iam_item_id.to_string(),
                to_rbum_item_id: to_iam_item_id.to_string(),
                to_own_paths: ctx.own_paths.to_string(),
                to_is_outside: false,
                ext: None,
            },
            attrs: vec![],
            envs: if start_timestamp.is_some() || end_timestamp.is_some() {
                vec![RbumRelEnvAggAddReq {
                    kind: RbumRelEnvKind::DatetimeRange,
                    value1: value1.to_string(),
                    value2: Some(value2.to_string()),
                }]
            } else {
                vec![]
            },
        };
        RbumRelServ::add_rel(req, funs, ctx).await?;
        if rel_kind == &IamRelKind::IamResRole {
            let iam_res = IamResServ::peek_item(
                from_iam_item_id,
                &IamResFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            if iam_res.kind != IamResKind::Api {
                return Ok(());
            }
            IamResCacheServ::add_or_modify_res_rel(
                &iam_res.code,
                &iam_res.method,
                &IamCacheResRelAddOrModifyReq {
                    st: if start_timestamp.is_some() { Some(value1) } else { None },
                    et: if end_timestamp.is_some() { Some(value2) } else { None },
                    accounts: vec![],
                    roles: vec![to_iam_item_id.to_string()],
                    groups: vec![],
                    apps: vec![],
                    tenants: vec![],
                },
                funs,
            )
            .await?;
        } else if rel_kind == &IamRelKind::IamResRole {
            let iam_res = IamResServ::peek_item(
                from_iam_item_id,
                &IamResFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            let rel_res = if iam_res.kind != IamResKind::Api {
                vec![iam_res]
            } else {
                let rel_res_api_ids = Self::find_to_id_rels(&IamRelKind::IamResApi, &from_iam_item_id, None, None, funs, ctx).await?;
                IamResServ::find_items(
                    &IamResFilterReq {
                        basic: RbumBasicFilterReq {
                            ids: Some(rel_res_api_ids),
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?
            };
            for res in rel_res {
                IamResCacheServ::add_or_modify_res_rel(
                    &res.code,
                    &res.method,
                    &IamCacheResRelAddOrModifyReq {
                        st: if start_timestamp.is_some() { Some(value1) } else { None },
                        et: if end_timestamp.is_some() { Some(value2) } else { None },
                        accounts: vec![],
                        roles: vec![to_iam_item_id.to_string()],
                        groups: vec![],
                        apps: vec![],
                        tenants: vec![],
                    },
                    funs,
                )
                .await?;
            }
        }
        Ok(())
    }

    pub async fn delete_simple_rel(rel_kind: &IamRelKind, from_iam_item_id: &str, to_iam_item_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
        let rel_ids = RbumRelServ::find_id_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(from_iam_item_id.to_string()),
                to_rbum_item_id: Some(to_iam_item_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if rel_ids.is_empty() {
            return Ok(());
        }
        for rel_id in rel_ids {
            RbumRelServ::delete_rbum(&rel_id, funs, ctx).await?;
        }
        match rel_kind {
            IamRelKind::IamResRole => {
                let iam_res = IamResServ::peek_item(
                    from_iam_item_id,
                    &IamResFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                if iam_res.kind != IamResKind::Api {
                    return Ok(());
                }
                IamResCacheServ::delete_res_rel(
                    &iam_res.code,
                    &iam_res.method,
                    &IamCacheResRelDeleteReq {
                        accounts: vec![],
                        roles: vec![to_iam_item_id.to_string()],
                        groups: vec![],
                        apps: vec![],
                        tenants: vec![],
                    },
                    funs,
                )
                .await?;
            }
            IamRelKind::IamResApi => {
                let iam_res = IamResServ::peek_item(
                    from_iam_item_id,
                    &IamResFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                if iam_res.kind != IamResKind::Api {
                    return Ok(());
                }
                let rel_menu_or_ele_res_ids = IamResServ::find_id_rel_roles(&IamRelKind::IamResApi, from_iam_item_id, true, None, None, funs, ctx).await?;
                let mut rel_roles_ids: HashMap<String, i64> = HashMap::new();
                for rel_menu_or_ele_res_id in rel_menu_or_ele_res_ids {
                    IamResServ::find_id_rel_roles(&IamRelKind::IamResRole, &rel_menu_or_ele_res_id, true, None, None, funs, ctx).await?.into_iter().for_each(|rel_role_id| {
                        rel_roles_ids.entry(rel_role_id).and_modify(|count| *count += 1).or_insert(0);
                    });
                }
                let only_one_rel_res_role_ids = rel_roles_ids.into_iter().filter(|(_, v)| *v == 1).map(|(k, _)| k).collect::<Vec<String>>();
                if only_one_rel_res_role_ids.is_empty() {
                    return Ok(());
                }
                for only_one_rel_res_role_id in only_one_rel_res_role_ids {
                    IamResCacheServ::delete_res_rel(
                        &iam_res.code,
                        &iam_res.method,
                        &IamCacheResRelDeleteReq {
                            accounts: vec![],
                            roles: vec![only_one_rel_res_role_id.to_string()],
                            groups: vec![],
                            apps: vec![],
                            tenants: vec![],
                        },
                        funs,
                    )
                    .await?;
                }
            }
            IamRelKind::IamAccountRole => {
                IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(from_iam_item_id, funs).await?;
            }
            IamRelKind::IamAccountApp => {
                IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(from_iam_item_id, funs).await?;
            }
        }
        Ok(())
    }

    pub async fn count_from_rels(rel_kind: &IamRelKind, with_sub: bool, from_iam_item_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<u64> {
        RbumRelServ::count_from_rels(&rel_kind.to_string(), &RbumRelFromKind::Item, with_sub, from_iam_item_id, funs, ctx).await
    }

    pub async fn find_from_id_rels(
        rel_kind: &IamRelKind,
        with_sub: bool,
        from_iam_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        RbumRelServ::find_from_id_rels(
            &rel_kind.to_string(),
            &RbumRelFromKind::Item,
            with_sub,
            from_iam_item_id,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn find_from_simple_rels(
        rel_kind: &IamRelKind,
        with_sub: bool,
        from_iam_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        RbumRelServ::find_from_simple_rels(
            &rel_kind.to_string(),
            &RbumRelFromKind::Item,
            with_sub,
            from_iam_item_id,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn paginate_from_id_rels(
        rel_kind: &IamRelKind,
        with_sub: bool,
        from_iam_item_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        RbumRelServ::paginate_from_id_rels(
            &rel_kind.to_string(),
            &RbumRelFromKind::Item,
            with_sub,
            from_iam_item_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn paginate_from_simple_rels(
        rel_kind: &IamRelKind,
        with_sub: bool,
        from_iam_item_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        RbumRelServ::paginate_from_simple_rels(
            &rel_kind.to_string(),
            &RbumRelFromKind::Item,
            with_sub,
            from_iam_item_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn count_to_rels(rel_kind: &IamRelKind, to_iam_item_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<u64> {
        RbumRelServ::count_to_rels(&rel_kind.to_string(), to_iam_item_id, funs, ctx).await
    }

    pub async fn find_to_id_rels(
        rel_kind: &IamRelKind,
        to_iam_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        RbumRelServ::find_to_id_rels(&rel_kind.to_string(), to_iam_item_id, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    pub async fn find_to_simple_rels(
        rel_kind: &IamRelKind,
        to_iam_item_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        RbumRelServ::find_to_simple_rels(&rel_kind.to_string(), to_iam_item_id, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }

    pub async fn paginate_to_id_rels(
        rel_kind: &IamRelKind,
        to_iam_item_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<String>> {
        RbumRelServ::paginate_to_id_rels(
            &rel_kind.to_string(),
            to_iam_item_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn paginate_to_simple_rels(
        rel_kind: &IamRelKind,
        to_iam_item_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        RbumRelServ::paginate_to_simple_rels(
            &rel_kind.to_string(),
            to_iam_item_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            ctx,
        )
        .await
    }

    pub async fn exist_rels(rel_kind: &IamRelKind, from_iam_item_id: &str, to_iam_item_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<bool> {
        RbumRelServ::exist_simple_rel(
            &RbumRelFindReq {
                tag: Some(rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(from_iam_item_id.to_string()),
                to_rbum_item_id: Some(to_iam_item_id.to_string()),
            },
            funs,
            ctx,
        )
        .await
    }
}
