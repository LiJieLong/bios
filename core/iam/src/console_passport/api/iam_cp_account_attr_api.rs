use std::collections::HashMap;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::OpenApi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_kind_attr_dto::RbumKindAttrSummaryResp;

use crate::basic::serv::iam_attr_serv::IamAttrServ;
use crate::iam_constants;

pub struct IamCpAccountAttrApi;

/// Passport Console Account Attr API
#[OpenApi(prefix_path = "/cp/account/attr", tag = "crate::iam_enumeration::Tag::Passport")]
impl IamCpAccountAttrApi {
    /// Find Account Attrs By Current Tenant
    #[oai(path = "/", method = "get")]
    async fn find_attrs(&self, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumKindAttrSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::find_account_attrs(&funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Find Account Ext Attr Values By Current Account
    #[oai(path = "/value", method = "get")]
    async fn find_account_attr_values(&self, ctx: TardisContextExtractor) -> TardisApiResult<HashMap<String, String>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::find_account_attr_values(&ctx.0.owner, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }
}
