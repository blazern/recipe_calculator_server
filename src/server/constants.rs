pub const CMD_REGISTER_DEVICE: &str = "/device/register_device";
pub const CMD_IS_DEVICE_REGISTERED: &str = "/device/device_registered";

pub const ARG_DEVICE_ID: &str = "device_id";

pub const FIELD_NAME_ERROR_DESCRIPTION: &str = "error_description";
pub const FIELD_NAME_STATUS: &str = "status";
pub const FIELD_NAME_REGISTERED: &str = "registered";
pub const FIELD_NAME_DEVICE_ID: &str = "device_id";

pub const FIELD_STATUS_OK: &str = "ok";
pub const FIELD_STATUS_CONNECTION_BROKEN: &str = "connection_broken";
pub const FIELD_STATUS_UNKNOWN_DEVICE: &str = "unknown_device";
pub const FIELD_STATUS_INTERNAL_ERROR: &str = "internal_error";
pub const FIELD_STATUS_INVALID_UUID: &str = "invalid_uuid";
pub const FIELD_STATUS_UNKNOWN_REQUEST: &str = "unknown_request";
pub const FIELD_STATUS_INVALID_QUERY: &str = "invalid_query";
pub const FIELD_STATUS_FOODSTUFF_DUPLICATION: &str = "foodstuff_duplication";
pub const FIELD_STATUS_HISTORY_ENTRY_DUPLICATION: &str = "history_entry_duplication";
pub const FIELD_STATUS_FOODSTUFF_NOT_FOUND: &str = "foodstuff_not_found";
pub const FIELD_STATUS_HISTORY_ENTRY_NOT_FOUND: &str = "history_entry_not_found";