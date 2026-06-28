use std::ffi::c_void;
use std::os::raw::{c_char, c_int, c_uint, c_ulong};
use std::ptr;

use super::types::{
    CompressionType, ConnectorBuffer, ConnectorConfig, ConnectorMode, ConnectorOperation,
    ConnectorResult, ConnectorState, ConnectorStats, DataEncoding, FeatureFlag,
};

static VERSION: &[u8] = b"windows-stub-connector-0.1\0";

#[export_name = "connector_init"]
pub unsafe extern "C" fn connector_init_stub(_config: *const ConnectorConfig) -> ConnectorResult {
    ConnectorResult::Success
}

#[export_name = "connector_shutdown"]
pub unsafe extern "C" fn connector_shutdown_stub() -> ConnectorResult {
    ConnectorResult::Success
}

#[export_name = "connector_drain"]
pub unsafe extern "C" fn connector_drain_stub() -> ConnectorResult {
    ConnectorResult::Success
}

#[export_name = "connector_get_config"]
pub unsafe extern "C" fn connector_get_config_stub(
    config: *mut ConnectorConfig,
) -> ConnectorResult {
    if config.is_null() {
        return ConnectorResult::ErrorInvalidParameter;
    }
    ptr::write(config, super::types::ConnectorConfigBuilder::new().build());
    ConnectorResult::Success
}

#[export_name = "connector_set_config"]
pub unsafe extern "C" fn connector_set_config_stub(
    _config: *const ConnectorConfig,
) -> ConnectorResult {
    ConnectorResult::Success
}

#[export_name = "connector_get_stats"]
pub unsafe extern "C" fn connector_get_stats_stub(stats: *mut ConnectorStats) -> ConnectorResult {
    if stats.is_null() {
        return ConnectorResult::ErrorInvalidParameter;
    }

    (*stats).struct_size = std::mem::size_of::<ConnectorStats>() as c_uint;
    (*stats).state = ConnectorState::Ready;
    (*stats).uptime_seconds = 0;
    (*stats).total_operations = 0;
    (*stats).successful_operations = 0;
    (*stats).failed_operations = 0;
    (*stats).timed_out_operations = 0;
    (*stats).retried_operations = 0;
    (*stats).bytes_sent = 0;
    (*stats).bytes_received = 0;
    (*stats).messages_sent = 0;
    (*stats).messages_received = 0;
    (*stats).active_connections = 0;
    (*stats).peak_connections = 0;
    (*stats).queue_depth = 0;
    (*stats).peak_queue_depth = 0;
    (*stats).average_latency_us = 0;
    (*stats).peak_latency_us = 0;
    (*stats).errors_by_type = [0; 32];
    (*stats).warnings_count = 0;
    (*stats).last_error_code = 0;
    (*stats).last_error_message = [0; 256];
    (*stats).reserved = [0; 16];

    ConnectorResult::Success
}

#[export_name = "connector_reset_stats"]
pub unsafe extern "C" fn connector_reset_stats_stub() -> ConnectorResult {
    ConnectorResult::Success
}

#[export_name = "connector_send"]
pub unsafe extern "C" fn connector_send_stub(buffer: *const ConnectorBuffer) -> ConnectorResult {
    if buffer.is_null() {
        ConnectorResult::ErrorInvalidParameter
    } else {
        ConnectorResult::Success
    }
}

#[export_name = "connector_receive"]
pub unsafe extern "C" fn connector_receive_stub(buffer: *mut ConnectorBuffer) -> ConnectorResult {
    if buffer.is_null() {
        return ConnectorResult::ErrorInvalidParameter;
    }
    (*buffer).size = 0;
    ConnectorResult::Success
}

#[export_name = "connector_submit"]
pub unsafe extern "C" fn connector_submit_stub(
    _operation: *mut ConnectorOperation,
) -> ConnectorResult {
    ConnectorResult::ErrorNotImplemented
}

#[export_name = "connector_cancel"]
pub unsafe extern "C" fn connector_cancel_stub(_operation_id: c_ulong) -> ConnectorResult {
    ConnectorResult::ErrorNotSupported
}

#[export_name = "connector_wait_all"]
pub unsafe extern "C" fn connector_wait_all_stub(_timeout_ms: c_uint) -> ConnectorResult {
    ConnectorResult::Success
}

#[export_name = "connector_buffer_alloc"]
pub unsafe extern "C" fn connector_buffer_alloc_stub(size: c_ulong) -> *mut ConnectorBuffer {
    let capacity = size as usize;
    let mut data = Vec::<u8>::with_capacity(capacity);
    let data_ptr = data.as_mut_ptr();
    std::mem::forget(data);

    Box::into_raw(Box::new(ConnectorBuffer {
        data: data_ptr.cast::<c_void>(),
        size: 0,
        capacity: size,
        offset: 0,
        encoding: DataEncoding::Binary,
        compression: CompressionType::None,
        checksum: 0,
        flags: 0,
        owner: 0,
    }))
}

#[export_name = "connector_buffer_free"]
pub unsafe extern "C" fn connector_buffer_free_stub(
    buffer: *mut ConnectorBuffer,
) -> ConnectorResult {
    if buffer.is_null() {
        return ConnectorResult::ErrorInvalidParameter;
    }

    let buffer = Box::from_raw(buffer);
    if !buffer.data.is_null() && buffer.capacity > 0 {
        let _ = Vec::from_raw_parts(buffer.data.cast::<u8>(), 0, buffer.capacity as usize);
    }

    ConnectorResult::Success
}

#[export_name = "connector_buffer_resize"]
pub unsafe extern "C" fn connector_buffer_resize_stub(
    buffer: *mut ConnectorBuffer,
    new_size: c_ulong,
) -> ConnectorResult {
    if buffer.is_null() {
        return ConnectorResult::ErrorInvalidParameter;
    }

    let old_capacity = (*buffer).capacity as usize;
    let old_size = (*buffer).size as usize;
    let old_data = (*buffer).data.cast::<u8>();
    let mut new_data = Vec::<u8>::with_capacity(new_size as usize);
    if !old_data.is_null() {
        ptr::copy_nonoverlapping(
            old_data,
            new_data.as_mut_ptr(),
            old_size.min(new_size as usize),
        );
        let _ = Vec::from_raw_parts(old_data, 0, old_capacity);
    }

    (*buffer).data = new_data.as_mut_ptr().cast::<c_void>();
    (*buffer).capacity = new_size;
    (*buffer).size = (*buffer).size.min(new_size);
    std::mem::forget(new_data);

    ConnectorResult::Success
}

#[export_name = "connector_buffer_reset"]
pub unsafe extern "C" fn connector_buffer_reset_stub(
    buffer: *mut ConnectorBuffer,
) -> ConnectorResult {
    if buffer.is_null() {
        return ConnectorResult::ErrorInvalidParameter;
    }
    (*buffer).offset = 0;
    (*buffer).size = 0;
    ConnectorResult::Success
}

#[export_name = "connector_version"]
pub unsafe extern "C" fn connector_version_stub() -> *const c_char {
    VERSION.as_ptr().cast::<c_char>()
}

#[export_name = "connector_has_feature"]
pub unsafe extern "C" fn connector_has_feature_stub(_feature: FeatureFlag) -> c_int {
    0
}

#[export_name = "connector_supported_features"]
pub unsafe extern "C" fn connector_supported_features_stub() -> c_uint {
    0
}

#[export_name = "connector_init_v1"]
pub unsafe extern "C" fn connector_init_v1_stub(
    _mode: ConnectorMode,
    _timeout_ms: c_uint,
    _max_connections: c_uint,
) -> ConnectorResult {
    ConnectorResult::Success
}

#[export_name = "connector_send_v1"]
pub unsafe extern "C" fn connector_send_v1_stub(
    data: *const c_void,
    _size: c_ulong,
    _timeout_ms: c_uint,
) -> ConnectorResult {
    if data.is_null() {
        ConnectorResult::ErrorInvalidParameter
    } else {
        ConnectorResult::Success
    }
}

#[export_name = "connector_receive_v1"]
pub unsafe extern "C" fn connector_receive_v1_stub(
    _buffer: *mut c_void,
    size: *mut c_ulong,
    _timeout_ms: c_uint,
) -> ConnectorResult {
    if !size.is_null() {
        *size = 0;
    }
    ConnectorResult::Success
}

#[export_name = "connector_get_stats_v1"]
pub unsafe extern "C" fn connector_get_stats_v1_stub(
    uptime: *mut c_ulong,
    operations: *mut c_ulong,
    errors: *mut c_ulong,
    bytes: *mut c_ulong,
) -> ConnectorResult {
    if !uptime.is_null() {
        *uptime = 0;
    }
    if !operations.is_null() {
        *operations = 0;
    }
    if !errors.is_null() {
        *errors = 0;
    }
    if !bytes.is_null() {
        *bytes = 0;
    }
    ConnectorResult::Success
}
