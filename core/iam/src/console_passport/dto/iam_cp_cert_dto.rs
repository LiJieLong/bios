use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCpUserPwdLoginReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub flag: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCpMailVCodeLoginGenVCodeReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCpMailVCodeLoginReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub vcode: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub flag: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCpPhoneVCodeLoginGenVCodeReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub phone: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCpPhoneVCodeLoginSendVCodeReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub phone: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub vcode: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub flag: Option<String>,
}
