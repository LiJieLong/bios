use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_basic::rbum::helper::rbum_scope_helper::get_path_item;
use bios_iam::basic::dto::iam_cert_dto::{IamUserPwdCertModifyReq, IamUserPwdCertRestReq};
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use bios_iam::iam_constants;
use bios_iam::iam_constants::{RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT, RBUM_SCOPE_LEVEL_TENANT};
use bios_iam::iam_enumeration::IamCertKind;

pub async fn test(
    sys_context: &TardisContext,
    t1_context: &TardisContext,
    t2_context: &TardisContext,
    t2_a1_context: &TardisContext,
    t2_a2_context: &TardisContext,
) -> TardisResult<()> {
    test_single_level(sys_context, RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT, t1_context).await?;
    test_single_level(t1_context, "bios", t2_context).await?;
    // test_single_level(t2_a1_context, "app_admin1", t2_a2_context).await?;
    Ok(())
}

async fn test_single_level(context: &TardisContext, ak: &str, another_context: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_cc_cert】 : test_single_level : Rest Password");
    let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(
        IamCertKind::UserPwd.to_string().as_str(),
        get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &context.own_paths),
        &funs,
    )
    .await?;
    assert!(IamCertUserPwdServ::reset_sk(
        &IamUserPwdCertRestReq {
            new_sk: TrimString("sssssssssss".to_string())
        },
        &another_context.owner,
        &rbum_cert_conf_id,
        &funs,
        context
    )
    .await
    .is_err());
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString(ak.to_string()),
            sk: TrimString("sssssssssss".to_string()),
            tenant_id: get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &context.own_paths),
            flag: None
        },
        &funs,
    )
    .await
    .is_err());
    IamCertUserPwdServ::reset_sk(
        &IamUserPwdCertRestReq {
            new_sk: TrimString("sssssssssss".to_string()),
        },
        &context.owner,
        &rbum_cert_conf_id,
        &funs,
        context,
    )
    .await?;
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString(ak.to_string()),
            sk: TrimString("sssssssssss".to_string()),
            tenant_id: get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &another_context.own_paths),
            flag: None
        },
        &funs,
    )
    .await
    .is_err());
    let account_info = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString(ak.to_string()),
            sk: TrimString("sssssssssss".to_string()),
            tenant_id: get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &context.own_paths),
            flag: None,
        },
        &funs,
    )
    .await?;

    info!("【test_cc_cert】 : test_single_level : Modify Cert");
    assert!(IamCpCertUserPwdServ::modify_cert_user_pwd(
        &another_context.owner,
        &IamUserPwdCertModifyReq {
            original_sk: TrimString("aaa".to_string()),
            new_sk: TrimString("123456789".to_string())
        },
        &funs,
        another_context
    )
    .await
    .is_err());
    assert!(IamCpCertUserPwdServ::modify_cert_user_pwd(
        &context.owner,
        &IamUserPwdCertModifyReq {
            original_sk: TrimString("aaa".to_string()),
            new_sk: TrimString("123456789".to_string())
        },
        &funs,
        context
    )
    .await
    .is_err());

    IamCpCertUserPwdServ::modify_cert_user_pwd(
        &context.owner,
        &IamUserPwdCertModifyReq {
            original_sk: TrimString("sssssssssss".to_string()),
            new_sk: TrimString("123456789".to_string()),
        },
        &funs,
        context,
    )
    .await?;

    IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString(ak.to_string()),
            sk: TrimString("123456789".to_string()),
            tenant_id: get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &context.own_paths),
            flag: None,
        },
        &funs,
    )
    .await?;

    funs.rollback().await?;
    Ok(())
}
