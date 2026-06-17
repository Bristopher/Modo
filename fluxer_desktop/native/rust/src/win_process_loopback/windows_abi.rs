// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ffi::c_void;

pub type Bool = i32;
pub type Byte = u8;
pub type Dword = u32;
pub type Handle = *mut c_void;
pub type Hresult = i32;
pub type Long = i32;
pub type LargeInteger = i64;
pub type ReferenceTime = i64;
pub type Uint = u32;
pub type Ulong = u32;
pub type Word = u16;
pub type Wchar = u16;

pub const FALSE: Bool = 0;
pub const TRUE: Bool = 1;
pub const INFINITE: Dword = 0xffff_ffff;
pub const INVALID_HANDLE_VALUE: Handle = usize::MAX as Handle;
pub const WAIT_FAILED: Dword = 0xffff_ffff;
pub const WAIT_OBJECT_0: Dword = 0;

pub const S_OK: Hresult = 0;
pub const E_NOINTERFACE: Hresult = 0x8000_4002_u32 as Hresult;

pub const COINIT_MULTITHREADED: Dword = 0;
pub const TH32CS_SNAPPROCESS: Dword = 0x0000_0002;
pub const VT_BLOB: Word = 65;
pub const WAVE_FORMAT_IEEE_FLOAT: Word = 3;
pub const WAVE_FORMAT_EXTENSIBLE: Word = 0xfffe;
pub const SPEAKER_FRONT_LEFT: Dword = 0x1;
pub const SPEAKER_FRONT_RIGHT: Dword = 0x2;
pub const KSAUDIO_SPEAKER_STEREO: Dword = SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT;

pub const AUDCLNT_BUFFERFLAGS_SILENT: Dword = 0x0000_0002;
pub const AUDCLNT_BUFFERFLAGS_TIMESTAMP_ERROR: Dword = 0x0000_0004;
pub const AUDCLNT_SHAREMODE_SHARED: AudioClientShareMode = 0;
pub const AUDCLNT_STREAMFLAGS_LOOPBACK: Dword = 0x0002_0000;
pub const AUDCLNT_STREAMFLAGS_EVENTCALLBACK: Dword = 0x0004_0000;
pub const AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM: Dword = 0x8000_0000;

pub type AudioClientActivationType = i32;
pub const AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK: AudioClientActivationType = 1;
pub type ProcessLoopbackMode = i32;
pub const PROCESS_LOOPBACK_MODE_INCLUDE_TARGET_PROCESS_TREE: ProcessLoopbackMode = 0;
pub const PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE: ProcessLoopbackMode = 1;
pub type AudioClientShareMode = i32;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

pub type Iid = Guid;

pub const IID_IUNKNOWN: Guid = Guid {
    data1: 0x0000_0000,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

pub const IID_IAGILE_OBJECT: Guid = Guid {
    data1: 0x94ea_2b94,
    data2: 0xe9cc,
    data3: 0x49e0,
    data4: [0xc0, 0xff, 0xee, 0x64, 0xca, 0x8f, 0x5b, 0x90],
};

pub const IID_IMARSHAL: Guid = Guid {
    data1: 0x0000_0003,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

pub const IID_IAUDIO_CLIENT: Guid = Guid {
    data1: 0x1cb9_ad4c,
    data2: 0xdbfa,
    data3: 0x4c32,
    data4: [0xb1, 0x78, 0xc2, 0xf5, 0x68, 0xa7, 0x03, 0xb2],
};

pub const IID_IAUDIO_CAPTURE_CLIENT: Guid = Guid {
    data1: 0xc8ad_bd64,
    data2: 0xe71e,
    data3: 0x48a0,
    data4: [0xa4, 0xde, 0x18, 0x5c, 0x39, 0x5c, 0xd3, 0x17],
};

pub const KSDATAFORMAT_SUBTYPE_IEEE_FLOAT: Guid = Guid {
    data1: 0x0000_0003,
    data2: 0x0000,
    data3: 0x0010,
    data4: [0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71],
};

pub const IID_IACTIVATE_AUDIO_INTERFACE_COMPLETION_HANDLER: Guid = Guid {
    data1: 0x41d9_49ab,
    data2: 0x9862,
    data3: 0x444a,
    data4: [0x80, 0xf6, 0xc2, 0x61, 0x33, 0x4d, 0xa5, 0xeb],
};

pub const VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK: &[u16] = &[
    'V' as u16,
    'A' as u16,
    'D' as u16,
    '\\' as u16,
    'P' as u16,
    'r' as u16,
    'o' as u16,
    'c' as u16,
    'e' as u16,
    's' as u16,
    's' as u16,
    '_' as u16,
    'L' as u16,
    'o' as u16,
    'o' as u16,
    'p' as u16,
    'b' as u16,
    'a' as u16,
    'c' as u16,
    'k' as u16,
    0,
];

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct WaveFormatEx {
    pub w_format_tag: Word,
    pub n_channels: Word,
    pub n_samples_per_sec: Dword,
    pub n_avg_bytes_per_sec: Dword,
    pub n_block_align: Word,
    pub w_bits_per_sample: Word,
    pub cb_size: Word,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union WaveFormatSamples {
    pub w_valid_bits_per_sample: Word,
    pub w_samples_per_block: Word,
    pub w_reserved: Word,
}

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct WaveFormatExtensible {
    pub format: WaveFormatEx,
    pub samples: WaveFormatSamples,
    pub dw_channel_mask: Dword,
    pub sub_format: Guid,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioClientProcessLoopbackParams {
    pub target_process_id: Dword,
    pub process_loopback_mode: ProcessLoopbackMode,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union AudioClientActivationParamsAnonymous {
    pub process_loopback_params: AudioClientProcessLoopbackParams,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AudioClientActivationParams {
    pub activation_type: AudioClientActivationType,
    pub anonymous: AudioClientActivationParamsAnonymous,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blob {
    pub cb_size: Ulong,
    pub p_blob_data: *mut Byte,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropVariant {
    pub vt: Word,
    pub w_reserved1: Word,
    pub w_reserved2: Word,
    pub w_reserved3: Word,
    pub blob: Blob,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcessEntry32W {
    pub dw_size: Dword,
    pub cnt_usage: Dword,
    pub th32_process_id: Dword,
    pub th32_default_heap_id: usize,
    pub th32_module_id: Dword,
    pub cnt_threads: Dword,
    pub th32_parent_process_id: Dword,
    pub pc_pri_class_base: Long,
    pub dw_flags: Dword,
    pub sz_exe_file: [Wchar; 260],
}

pub type NapiEnv = *mut c_void;
pub type NapiValue = *mut c_void;
pub type NapiCallbackInfo = *mut c_void;
pub type NapiThreadsafeFunction = *mut c_void;
pub type NapiStatus = i32;
pub type NapiValueType = i32;
pub type NapiTypedArrayType = i32;
pub type NapiThreadsafeFunctionReleaseMode = i32;
pub type NapiThreadsafeFunctionCallMode = i32;
pub type NapiPropertyAttributes = i32;

pub const NAPI_OK: NapiStatus = 0;
pub const NAPI_UNDEFINED: NapiValueType = 0;
pub const NAPI_NULL: NapiValueType = 1;
pub const NAPI_BOOLEAN: NapiValueType = 2;
pub const NAPI_NUMBER: NapiValueType = 3;
pub const NAPI_STRING: NapiValueType = 4;
pub const NAPI_FLOAT32_ARRAY: NapiTypedArrayType = 6;
pub const NAPI_DEFAULT_METHOD: NapiPropertyAttributes = 5;
pub const NAPI_TSFN_NONBLOCKING: NapiThreadsafeFunctionCallMode = 0;
pub const NAPI_TSFN_RELEASE: NapiThreadsafeFunctionReleaseMode = 0;
pub const NAPI_TSFN_ABORT: NapiThreadsafeFunctionReleaseMode = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_format_extensible_layout_matches_windows_abi() {
        assert_eq!(18, std::mem::size_of::<WaveFormatEx>());
        assert_eq!(40, std::mem::size_of::<WaveFormatExtensible>());
        assert_eq!(18, std::mem::offset_of!(WaveFormatExtensible, samples));
        assert_eq!(
            20,
            std::mem::offset_of!(WaveFormatExtensible, dw_channel_mask)
        );
        assert_eq!(24, std::mem::offset_of!(WaveFormatExtensible, sub_format));
    }

    #[test]
    fn stereo_float_extensible_constants_match_ksmedia_h() {
        assert_eq!(0xfffe, WAVE_FORMAT_EXTENSIBLE);
        assert_eq!(0x3, KSAUDIO_SPEAKER_STEREO);
        assert_eq!(0x4, AUDCLNT_BUFFERFLAGS_TIMESTAMP_ERROR);
        assert_eq!(0x0000_0003, KSDATAFORMAT_SUBTYPE_IEEE_FLOAT.data1);
        assert_eq!(0x0000, KSDATAFORMAT_SUBTYPE_IEEE_FLOAT.data2);
        assert_eq!(0x0010, KSDATAFORMAT_SUBTYPE_IEEE_FLOAT.data3);
        assert_eq!(
            [0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71],
            KSDATAFORMAT_SUBTYPE_IEEE_FLOAT.data4
        );
    }

    #[test]
    fn iagile_object_iid_matches_objidlbase_h() {
        assert_eq!(0x94ea_2b94, IID_IAGILE_OBJECT.data1);
        assert_eq!(0xe9cc, IID_IAGILE_OBJECT.data2);
        assert_eq!(0x49e0, IID_IAGILE_OBJECT.data3);
        assert_eq!(
            [0xc0, 0xff, 0xee, 0x64, 0xca, 0x8f, 0x5b, 0x90],
            IID_IAGILE_OBJECT.data4
        );
    }

    #[test]
    fn imarshal_iid_matches_objidlbase_h() {
        assert_eq!(0x0000_0003, IID_IMARSHAL.data1);
        assert_eq!(0x0000, IID_IMARSHAL.data2);
        assert_eq!(0x0000, IID_IMARSHAL.data3);
        assert_eq!(
            [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
            IID_IMARSHAL.data4
        );
    }

    #[test]
    fn node_api_value_type_constants_match_node_api_h() {
        assert_eq!(0, NAPI_UNDEFINED);
        assert_eq!(1, NAPI_NULL);
        assert_eq!(2, NAPI_BOOLEAN);
        assert_eq!(3, NAPI_NUMBER);
        assert_eq!(4, NAPI_STRING);
    }

    #[test]
    fn process_loopback_device_id_is_utf16_null_terminated() {
        assert_eq!(Some(&0), VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK.last());
        let without_nul = &VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK
            [..VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK.len() - 1];
        assert_eq!(
            "VAD\\Process_Loopback",
            String::from_utf16(without_nul).expect("utf16")
        );
    }
}
