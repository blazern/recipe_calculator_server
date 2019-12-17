pub const CMD_REGISTER_USER: &str = "/v1/user/register";

pub const ARG_USER_NAME: &str = "name";
pub const ARG_SOCIAL_NETWORK_TYPE: &str = "social_network_type";
pub const ARG_SOCIAL_NETWORK_TOKEN: &str = "social_network_token";
pub const ARG_OVERRIDES: &str = "overrides";

pub const FIELD_NAME_ERROR_DESCRIPTION: &str = "error_description";
pub const FIELD_NAME_STATUS: &str = "status";
pub const FIELD_NAME_REGISTERED: &str = "registered";
pub const FIELD_NAME_USER_ID: &str = "device_id";
pub const FIELD_NAME_CLIENT_TOKEN: &str = "client_token";

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
