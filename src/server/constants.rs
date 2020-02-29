pub const CMD_REGISTER_USER: &str = "/v1/user/register";
pub const CMD_UPDATE_FCM_TOKEN: &str = "/v1/user/update_fcm_token";
pub const CMD_START_PAIRING: &str = "/v1/user/start_pairing";
pub const CMD_PAIRING_REQUEST: &str = "/v1/user/pairing_request";
pub const CMD_MOVE_DEVICE_ACCOUNT: &str = "/v1/user/move_device_account";
pub const CMD_LIST_PARTNERS: &str = "/v1/user/list_partners";
pub const CMD_UNPAIR: &str = "/v1/user/unpair";
pub const CMD_DIRECT_PARTNER_MSG: &str = "/v1/user/direct_partner_msg";

pub const ARG_USER_NAME: &str = "name";
pub const ARG_SOCIAL_NETWORK_TYPE: &str = "social_network_type";
pub const ARG_SOCIAL_NETWORK_TOKEN: &str = "social_network_token";
pub const ARG_OVERRIDES: &str = "overrides";
pub const ARG_CLIENT_TOKEN: &str = "client_token";
pub const ARG_USER_ID: &str = "user_id";
pub const ARG_FCM_TOKEN: &str = "fcm_token";
pub const ARG_PARTNER_PAIRING_CODE: &str = "partner_pairing_code";
pub const ARG_PARTNER_USER_ID: &str = "partner_user_id";

pub const FIELD_NAME_ERROR_DESCRIPTION: &str = "error_description";
pub const FIELD_NAME_STATUS: &str = "status";
pub const FIELD_NAME_REGISTERED: &str = "registered";
pub const FIELD_NAME_USER_ID: &str = "user_id";
pub const FIELD_NAME_USER_NAME: &str = "user_name";
pub const FIELD_NAME_CLIENT_TOKEN: &str = "client_token";
pub const FIELD_NAME_PAIRING_CODE_EXPIRATION_DATE: &str = "pairing_code_expiration_date";
pub const FIELD_NAME_PAIRING_CODE: &str = "pairing_code";
pub const FIELD_NAME_PARTNER_USER_ID: &str = "partner_user_id";
pub const FIELD_NAME_PARTNER_NAME: &str = "partner_name";
pub const FIELD_NAME_PARTNERS: &str = "partners";

pub const FIELD_STATUS_OK: &str = "ok";
pub const FIELD_STATUS_CONNECTION_BROKEN: &str = "connection_broken";
pub const FIELD_STATUS_INTERNAL_ERROR: &str = "internal_error";
pub const FIELD_STATUS_PARAM_MISSING: &str = "param_missing";
pub const FIELD_STATUS_INVALID_UUID: &str = "invalid_uuid";
pub const FIELD_STATUS_UNKNOWN_REQUEST: &str = "unknown_request";
pub const FIELD_STATUS_INVALID_QUERY: &str = "invalid_query";
pub const FIELD_STATUS_FOODSTUFF_DUPLICATION: &str = "foodstuff_duplication";
pub const FIELD_STATUS_HISTORY_ENTRY_DUPLICATION: &str = "history_entry_duplication";
pub const FIELD_STATUS_FOODSTUFF_NOT_FOUND: &str = "foodstuff_not_found";
pub const FIELD_STATUS_HISTORY_ENTRY_NOT_FOUND: &str = "history_entry_not_found";
pub const FIELD_STATUS_ALREADY_REGISTERED: &str = "already_registered";
pub const FIELD_STATUS_TOKEN_CHECK_FAIL: &str = "token_check_fail";
pub const FIELD_STATUS_USER_NOT_FOUND: &str = "user_not_found";
pub const FIELD_STATUS_INVALID_CLIENT_TOKEN: &str = "invalid_client_token";
pub const FIELD_STATUS_PARTNER_USER_NOT_FOUND: &str = "partner_user_not_found";
pub const FIELD_STATUS_INVALID_PARTNER_PAIRING_CODE: &str = "invalid_partner_pairing_code";

pub const SERV_FIELD_MSG_TYPE: &str = "msg_type";
pub const SERV_FIELD_PAIRING_PARTNER_USER_ID: &str = "pairing_partner_user_id";
pub const SERV_FIELD_PARTNER_USER_ID: &str = "partner_user_id";
pub const SERV_FIELD_PARTNER_NAME: &str = "partner_name";
pub const SERV_FIELD_REQUEST_EXPIRATION_DATE: &str = "request_expiration_date";
pub const SERV_FIELD_MSG: &str = "msg";

pub const SERV_MSG_PAIRING_REQUEST_FROM_PARTNER: &str = "pairing_request_from_partner";
pub const SERV_MSG_PAIRED_WITH_PARTNER: &str = "paired_with_partner";
pub const SERV_MSG_DIRECT_MSG_FROM_PARTNER: &str = "direct_msg_from_partner";

pub const PAIRING_CODES_FAMILY_NAME: &str = "default";
