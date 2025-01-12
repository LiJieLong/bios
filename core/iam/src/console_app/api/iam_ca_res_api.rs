use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::dto::rbum_set_cate_dto::RbumSetTreeResp;

use crate::basic::serv::iam_res_serv::IamResServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRelKind;

pub struct IamCaResApi;

/// App Console Res API
///
/// Note: the current res only supports sys level.
#[OpenApi(prefix_path = "/ca/res", tag = "crate::iam_enumeration::Tag::App")]
impl IamCaResApi {
    /// Find Res Tree
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(&self, sys_res: Query<Option<bool>>, parent_cate_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetTreeResp>> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = if sys_res.0.unwrap_or(false) {
            IamSetServ::get_set_id_by_code(&IamSetServ::get_default_res_code_by_own_paths(""), true, &funs, &ctx.0).await?
        } else {
            IamSetServ::get_default_set_id_by_ctx(false, &funs, &ctx.0).await?
        };
        let result = IamSetServ::get_tree(&set_id, parent_cate_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Find Rel Roles By Res Id
    #[oai(path = "/:id/role", method = "get")]
    async fn find_rel_roles(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::find_simple_rel_roles(&IamRelKind::IamResRole, &id.0, false, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }
}
