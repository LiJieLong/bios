use tardis::basic::result::TardisResult;

pub const RBUM_KIND_SCHEME_IAM_TENANT: &str = "iam_tenant";
pub const RBUM_KIND_SCHEME_IAM_APP: &str = "iam_app";
pub const RBUM_KIND_SCHEME_IAM_ACCOUNT: &str = "iam_account";
pub const RBUM_KIND_SCHEME_IAM_ROLE: &str = "iam_role";
pub const RBUM_KIND_SCHEME_IAM_RES_HTTP: &str = "iam_res_http";

pub const RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT: &str = "bios";
pub const RBUM_ITEM_NAME_SYS_ADMIN_ROLE: &str = "sys_admin";
pub const RBUM_ITEM_NAME_TENANT_ADMIN_ROLE: &str = "tenant_admin";
pub const RBUM_ITEM_NAME_APP_ADMIN_ROLE: &str = "app_admin";

pub const RBUM_ITEM_ID_TENANT_LEN: usize = 6;
pub const RBUM_ITEM_ID_APP_LEN: usize = 6;

pub const RBUM_SCOPE_LEVEL_GLOBAL: i32 = 0;
pub const RBUM_SCOPE_LEVEL_TENANT: i32 = 1;
pub const RBUM_SCOPE_LEVEL_APP: i32 = 2;

static mut BASIC_INFO: BasicInfo = BasicInfo { info: None };

#[derive(Debug)]
struct BasicInfo {
    pub info: Option<BasicInfoPub>,
}

#[derive(Debug)]
pub struct BasicInfoPub {
    pub kind_tenant_id: String,
    pub kind_app_id: String,
    pub kind_account_id: String,
    pub kind_role_id: String,
    pub kind_http_res_id: String,
    pub domain_iam_id: String,
    pub role_sys_admin_id: String,
    pub role_tenant_admin_id: String,
    pub role_app_admin_id: String,
}

pub fn set_basic_info(basic_info: BasicInfoPub) -> TardisResult<()> {
    unsafe {
        BASIC_INFO.info = Some(basic_info);
    }
    Ok(())
}

pub fn get_rbum_basic_info() -> &'static BasicInfoPub {
    unsafe {
        match BASIC_INFO.info {
            Some(ref info) => info,
            None => panic!("Basic info not set"),
        }
    }
}
