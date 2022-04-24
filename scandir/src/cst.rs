/// Read only file- and directory names without metadata
pub const RETURN_TYPE_FAST: u8 = 0;
/// Read also basic metadata
pub const RETURN_TYPE_BASE: u8 = 1;
/// Read also extended metadata
pub const RETURN_TYPE_EXT: u8 = 2;
/// Also provide relative path and filename in result
pub const RETURN_TYPE_FULL: u8 = 3;
pub const RETURN_TYPE_WALK: u8 = 4;
