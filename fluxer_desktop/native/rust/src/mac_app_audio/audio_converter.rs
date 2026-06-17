// SPDX-License-Identifier: AGPL-3.0-or-later

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioStreamBasicDescription {
    pub m_sample_rate: f64,
    pub m_format_id: u32,
    pub m_format_flags: u32,
    pub m_bytes_per_packet: u32,
    pub m_frames_per_packet: u32,
    pub m_bytes_per_frame: u32,
    pub m_channels_per_frame: u32,
    pub m_bits_per_channel: u32,
    pub m_reserved: u32,
}

pub const K_AUDIO_FORMAT_LINEAR_PCM: u32 =
    (('l' as u32) << 24) | (('p' as u32) << 16) | (('c' as u32) << 8) | ('m' as u32);

pub const K_LINEAR_PCM_FORMAT_FLAG_IS_FLOAT: u32 = 1 << 0;
pub const K_LINEAR_PCM_FORMAT_FLAG_IS_BIG_ENDIAN: u32 = 1 << 1;
pub const K_LINEAR_PCM_FORMAT_FLAG_IS_SIGNED_INTEGER: u32 = 1 << 2;
pub const K_LINEAR_PCM_FORMAT_FLAG_IS_PACKED: u32 = 1 << 3;
pub const K_LINEAR_PCM_FORMAT_FLAG_IS_NON_INTERLEAVED: u32 = 1 << 5;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AudioBuffer {
    pub m_number_channels: u32,
    pub m_data_byte_size: u32,
    pub m_data: *const u8,
}

impl AudioBuffer {
    pub fn from_slice<T>(channels: u32, slice: &[T]) -> Self {
        Self {
            m_number_channels: channels,
            m_data_byte_size: std::mem::size_of_val(slice) as u32,
            m_data: slice.as_ptr().cast(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AudioBufferListN<const N: usize> {
    pub m_number_buffers: u32,
    pub buffers: [AudioBuffer; N],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcmConvertError {
    UnsupportedFormat,
    UnsupportedBitDepth,
    MissingData,
    OutputTooSmall,
}

pub fn build_output_asbd(sample_rate: f64, channels: u32) -> AudioStreamBasicDescription {
    let bytes_per_frame = 4 * channels;
    AudioStreamBasicDescription {
        m_sample_rate: sample_rate,
        m_format_id: K_AUDIO_FORMAT_LINEAR_PCM,
        m_format_flags: K_LINEAR_PCM_FORMAT_FLAG_IS_FLOAT | K_LINEAR_PCM_FORMAT_FLAG_IS_PACKED,
        m_bytes_per_packet: bytes_per_frame,
        m_frames_per_packet: 1,
        m_bytes_per_frame: bytes_per_frame,
        m_channels_per_frame: channels,
        m_bits_per_channel: 32,
        m_reserved: 0,
    }
}

pub fn build_input_asbd(
    sample_rate: f64,
    channels: u32,
    non_interleaved: bool,
) -> AudioStreamBasicDescription {
    let bytes_per_frame = if non_interleaved { 4 } else { 4 * channels };
    let mut flags = K_LINEAR_PCM_FORMAT_FLAG_IS_FLOAT | K_LINEAR_PCM_FORMAT_FLAG_IS_PACKED;
    if non_interleaved {
        flags |= K_LINEAR_PCM_FORMAT_FLAG_IS_NON_INTERLEAVED;
    }
    AudioStreamBasicDescription {
        m_sample_rate: sample_rate,
        m_format_id: K_AUDIO_FORMAT_LINEAR_PCM,
        m_format_flags: flags,
        m_bytes_per_packet: bytes_per_frame,
        m_frames_per_packet: 1,
        m_bytes_per_frame: bytes_per_frame,
        m_channels_per_frame: channels,
        m_bits_per_channel: 32,
        m_reserved: 0,
    }
}

pub fn output_frame_capacity(input_frames: u32, in_rate: f64, out_rate: f64) -> u32 {
    if in_rate <= 0.0 {
        return input_frames;
    }
    ((input_frames as f64 * (out_rate / in_rate)).ceil() + 1.0) as u32
}

pub fn converted_frame_count(input_frames: u32, in_rate: f64, out_rate: f64) -> u32 {
    if input_frames == 0 {
        return 0;
    }
    if in_rate <= 0.0 || out_rate <= 0.0 || in_rate == out_rate {
        return input_frames;
    }
    (input_frames as f64 * (out_rate / in_rate)).ceil() as u32
}

pub fn is_linear_pcm(asbd: AudioStreamBasicDescription) -> bool {
    asbd.m_format_id == K_AUDIO_FORMAT_LINEAR_PCM
}

pub fn is_float(asbd: AudioStreamBasicDescription) -> bool {
    (asbd.m_format_flags & K_LINEAR_PCM_FORMAT_FLAG_IS_FLOAT) != 0
}

pub fn is_signed_integer(asbd: AudioStreamBasicDescription) -> bool {
    (asbd.m_format_flags & K_LINEAR_PCM_FORMAT_FLAG_IS_SIGNED_INTEGER) != 0
}

pub fn is_big_endian(asbd: AudioStreamBasicDescription) -> bool {
    (asbd.m_format_flags & K_LINEAR_PCM_FORMAT_FLAG_IS_BIG_ENDIAN) != 0
}

pub fn is_packed(asbd: AudioStreamBasicDescription) -> bool {
    (asbd.m_format_flags & K_LINEAR_PCM_FORMAT_FLAG_IS_PACKED) != 0
}

pub fn is_non_interleaved(asbd: AudioStreamBasicDescription) -> bool {
    (asbd.m_format_flags & K_LINEAR_PCM_FORMAT_FLAG_IS_NON_INTERLEAVED) != 0
}

pub fn is_native_f32_interleaved(asbd: AudioStreamBasicDescription) -> bool {
    is_linear_pcm(asbd)
        && is_float(asbd)
        && asbd.m_bits_per_channel == 32
        && is_packed(asbd)
        && !is_big_endian(asbd)
        && !is_non_interleaved(asbd)
}

pub fn is_native_f32_planar(asbd: AudioStreamBasicDescription) -> bool {
    is_linear_pcm(asbd)
        && is_float(asbd)
        && asbd.m_bits_per_channel == 32
        && is_packed(asbd)
        && !is_big_endian(asbd)
        && is_non_interleaved(asbd)
}

fn bytes_per_sample(asbd: AudioStreamBasicDescription) -> Result<u32, PcmConvertError> {
    if !is_linear_pcm(asbd) {
        return Err(PcmConvertError::UnsupportedFormat);
    }
    if asbd.m_bits_per_channel == 0 || !asbd.m_bits_per_channel.is_multiple_of(8) {
        return Err(PcmConvertError::UnsupportedBitDepth);
    }
    let bytes = asbd.m_bits_per_channel / 8;
    if matches!(bytes, 1 | 2 | 3 | 4 | 8) {
        Ok(bytes)
    } else {
        Err(PcmConvertError::UnsupportedBitDepth)
    }
}

pub fn input_frame_count_for_buffer_list<const N: usize>(
    asbd: AudioStreamBasicDescription,
    list: &AudioBufferListN<N>,
) -> Result<u32, PcmConvertError> {
    if !is_linear_pcm(asbd) {
        return Err(PcmConvertError::UnsupportedFormat);
    }
    if list.m_number_buffers == 0 {
        return Err(PcmConvertError::MissingData);
    }
    let first = &list.buffers[0];
    if first.m_data.is_null() || first.m_data_byte_size == 0 {
        return Ok(0);
    }
    let channels = if asbd.m_channels_per_frame == 0 {
        1
    } else {
        asbd.m_channels_per_frame
    };
    let bps = bytes_per_sample(asbd)?;
    let frame_stride = if asbd.m_bytes_per_frame != 0 {
        asbd.m_bytes_per_frame
    } else if is_non_interleaved(asbd) {
        bps
    } else {
        bps * channels
    };
    if frame_stride == 0 {
        return Ok(0);
    }
    Ok(first.m_data_byte_size / frame_stride)
}

fn read_unsigned(bytes: &[u8], big_endian: bool) -> u64 {
    if big_endian {
        bytes
            .iter()
            .fold(0_u64, |out, byte| (out << 8) | *byte as u64)
    } else {
        bytes.iter().enumerate().fold(0_u64, |out, (index, byte)| {
            out | ((*byte as u64) << (index * 8))
        })
    }
}

fn sign_extend(value: u64, bits: u32) -> i64 {
    if bits == 64 {
        return value as i64;
    }
    let shift = 64 - bits;
    ((value << shift) as i64) >> shift
}

fn pow2_float(exponent: u32) -> f64 {
    2_f64.powi(exponent as i32)
}

fn read_scalar_sample(
    asbd: AudioStreamBasicDescription,
    bytes: &[u8],
) -> Result<f32, PcmConvertError> {
    let bits = asbd.m_bits_per_channel;
    if bits == 0 || bits > 64 || !bits.is_multiple_of(8) {
        return Err(PcmConvertError::UnsupportedBitDepth);
    }
    let raw = read_unsigned(bytes, is_big_endian(asbd));
    if is_float(asbd) {
        return match bits {
            32 => Ok(f32::from_bits(raw as u32)),
            64 => Ok(f64::from_bits(raw) as f32),
            _ => Err(PcmConvertError::UnsupportedBitDepth),
        };
    }
    if is_signed_integer(asbd) {
        let signed = sign_extend(raw, bits);
        return Ok((signed as f64 / pow2_float(bits - 1)) as f32);
    }
    let midpoint = pow2_float(bits - 1);
    Ok(((raw as f64 - midpoint) / midpoint) as f32)
}

fn source_channel_for(target_channel: u32, source_channels: u32) -> u32 {
    if source_channels <= 1 {
        0
    } else {
        target_channel.min(source_channels - 1)
    }
}

fn read_frame_channel<const N: usize>(
    asbd: AudioStreamBasicDescription,
    list: &AudioBufferListN<N>,
    frame_index: u32,
    target_channel: u32,
) -> Result<f32, PcmConvertError> {
    let channels = if asbd.m_channels_per_frame == 0 {
        1
    } else {
        asbd.m_channels_per_frame
    };
    let source_channel = source_channel_for(target_channel, channels);
    let bps = bytes_per_sample(asbd)?;
    let non_interleaved = is_non_interleaved(asbd);
    let buffer_count = (list.m_number_buffers as usize).min(N);
    if buffer_count == 0 {
        return Err(PcmConvertError::MissingData);
    }
    let buffer_index = if non_interleaved && buffer_count > 1 {
        (source_channel as usize).min(buffer_count - 1)
    } else {
        0
    };
    let buffer = &list.buffers[buffer_index];
    if buffer.m_data.is_null() {
        return Err(PcmConvertError::MissingData);
    }
    let data =
        unsafe { std::slice::from_raw_parts(buffer.m_data, buffer.m_data_byte_size as usize) };
    let frame_stride = if asbd.m_bytes_per_frame != 0 {
        asbd.m_bytes_per_frame
    } else if non_interleaved {
        bps
    } else {
        bps * channels
    };
    let channel_offset = if non_interleaved && buffer_count > 1 {
        0
    } else {
        source_channel * bps
    };
    let offset = frame_index as usize * frame_stride as usize + channel_offset as usize;
    let end = offset + bps as usize;
    if end > data.len() {
        return Err(PcmConvertError::MissingData);
    }
    read_scalar_sample(asbd, &data[offset..end])
}

fn aligned_f32_slice(
    buffer: &AudioBuffer,
    samples: usize,
) -> Result<Option<&[f32]>, PcmConvertError> {
    if buffer.m_data.is_null() {
        return Err(PcmConvertError::MissingData);
    }
    let bytes = samples * std::mem::size_of::<f32>();
    if (buffer.m_data_byte_size as usize) < bytes {
        return Err(PcmConvertError::MissingData);
    }
    let address = buffer.m_data as usize;
    if !address.is_multiple_of(std::mem::align_of::<f32>()) {
        return Ok(None);
    }
    Ok(Some(unsafe {
        std::slice::from_raw_parts(buffer.m_data.cast::<f32>(), samples)
    }))
}

fn copy_native_interleaved_f32<const N: usize>(
    asbd: AudioStreamBasicDescription,
    list: &AudioBufferListN<N>,
    input_frames: u32,
    output_channels: u32,
    output: &mut [f32],
) -> Result<Option<u32>, PcmConvertError> {
    let source_channels = asbd.m_channels_per_frame.max(1);
    let source_stride = source_channels * std::mem::size_of::<f32>() as u32;
    if asbd.m_bytes_per_frame != 0 && asbd.m_bytes_per_frame != source_stride {
        return Ok(None);
    }
    if list.m_number_buffers == 0 || N == 0 {
        return Err(PcmConvertError::MissingData);
    }
    let source_samples = input_frames as usize * source_channels as usize;
    let output_samples = input_frames as usize * output_channels as usize;
    let Some(source) = aligned_f32_slice(&list.buffers[0], source_samples)? else {
        return Ok(None);
    };
    assert!(output_samples <= output.len());
    if source_channels == output_channels {
        output[..output_samples].copy_from_slice(&source[..output_samples]);
        return Ok(Some(input_frames));
    }
    for frame in 0..input_frames as usize {
        let source_base = frame * source_channels as usize;
        let output_base = frame * output_channels as usize;
        for channel in 0..output_channels as usize {
            let source_channel = source_channel_for(channel as u32, source_channels) as usize;
            output[output_base + channel] = source[source_base + source_channel];
        }
    }
    Ok(Some(input_frames))
}

fn copy_native_planar_f32<const N: usize>(
    asbd: AudioStreamBasicDescription,
    list: &AudioBufferListN<N>,
    input_frames: u32,
    output_channels: u32,
    output: &mut [f32],
) -> Result<Option<u32>, PcmConvertError> {
    if asbd.m_bytes_per_frame != 0 && asbd.m_bytes_per_frame != std::mem::size_of::<f32>() as u32 {
        return Ok(None);
    }
    let source_channels = asbd.m_channels_per_frame.max(1);
    let buffer_count = (list.m_number_buffers as usize).min(N);
    if buffer_count == 0 {
        return Err(PcmConvertError::MissingData);
    }
    let output_samples = input_frames as usize * output_channels as usize;
    assert!(output_samples <= output.len());
    for channel in 0..output_channels as usize {
        let source_channel = source_channel_for(channel as u32, source_channels) as usize;
        let buffer_index = if buffer_count > 1 {
            source_channel.min(buffer_count - 1)
        } else {
            0
        };
        let Some(source) = aligned_f32_slice(&list.buffers[buffer_index], input_frames as usize)?
        else {
            return Ok(None);
        };
        for frame in 0..input_frames as usize {
            output[frame * output_channels as usize + channel] = source[frame];
        }
    }
    Ok(Some(input_frames))
}

fn copy_native_f32<const N: usize>(
    asbd: AudioStreamBasicDescription,
    list: &AudioBufferListN<N>,
    input_frames: u32,
    output_channels: u32,
    output: &mut [f32],
) -> Result<Option<u32>, PcmConvertError> {
    if is_native_f32_interleaved(asbd) {
        return copy_native_interleaved_f32(asbd, list, input_frames, output_channels, output);
    }
    if is_native_f32_planar(asbd) {
        return copy_native_planar_f32(asbd, list, input_frames, output_channels, output);
    }
    Ok(None)
}

pub fn convert_buffer_list_to_interleaved_f32<const N: usize>(
    asbd: AudioStreamBasicDescription,
    list: &AudioBufferListN<N>,
    input_frames: u32,
    output_sample_rate: f64,
    output_channels: u32,
    output: &mut [f32],
) -> Result<u32, PcmConvertError> {
    if !is_linear_pcm(asbd) {
        return Err(PcmConvertError::UnsupportedFormat);
    }
    if output_channels == 0 {
        return Ok(0);
    }
    let out_frames = converted_frame_count(input_frames, asbd.m_sample_rate, output_sample_rate);
    let needed = out_frames as usize * output_channels as usize;
    if needed > output.len() {
        return Err(PcmConvertError::OutputTooSmall);
    }
    if input_frames == 0 || out_frames == 0 {
        return Ok(0);
    }
    let native_frames = if asbd.m_sample_rate == output_sample_rate {
        copy_native_f32(asbd, list, input_frames, output_channels, output)?
    } else {
        None
    };
    if let Some(frames) = native_frames {
        return Ok(frames);
    }
    let step = if asbd.m_sample_rate > 0.0 && output_sample_rate > 0.0 {
        asbd.m_sample_rate / output_sample_rate
    } else {
        1.0
    };
    for out_frame in 0..out_frames {
        let src_pos = out_frame as f64 * step;
        let mut base = src_pos.floor() as u32;
        if base >= input_frames {
            base = input_frames - 1;
        }
        let next = if base + 1 < input_frames {
            base + 1
        } else {
            base
        };
        let frac = (src_pos - src_pos.floor()) as f32;
        for ch in 0..output_channels {
            let a = read_frame_channel(asbd, list, base, ch)?;
            let b = read_frame_channel(asbd, list, next, ch)?;
            output[out_frame as usize * output_channels as usize + ch as usize] =
                a + (b - a) * frac;
        }
    }
    Ok(out_frames)
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use super::*;

    #[test]
    fn audio_stream_basic_description_field_offsets_match_apple_abi() {
        assert_eq!(0, offset_of!(AudioStreamBasicDescription, m_sample_rate));
        assert_eq!(8, offset_of!(AudioStreamBasicDescription, m_format_id));
        assert_eq!(12, offset_of!(AudioStreamBasicDescription, m_format_flags));
        assert_eq!(
            16,
            offset_of!(AudioStreamBasicDescription, m_bytes_per_packet)
        );
        assert_eq!(
            20,
            offset_of!(AudioStreamBasicDescription, m_frames_per_packet)
        );
        assert_eq!(
            24,
            offset_of!(AudioStreamBasicDescription, m_bytes_per_frame)
        );
        assert_eq!(
            28,
            offset_of!(AudioStreamBasicDescription, m_channels_per_frame)
        );
        assert_eq!(
            32,
            offset_of!(AudioStreamBasicDescription, m_bits_per_channel)
        );
        assert_eq!(36, offset_of!(AudioStreamBasicDescription, m_reserved));
        assert_eq!(40, size_of::<AudioStreamBasicDescription>());
    }

    #[test]
    fn audio_buffer_offsets() {
        assert_eq!(0, offset_of!(AudioBuffer, m_number_channels));
        assert_eq!(4, offset_of!(AudioBuffer, m_data_byte_size));
        assert_eq!(8, offset_of!(AudioBuffer, m_data));
        assert_eq!(16, size_of::<AudioBuffer>());
        assert_eq!(0, offset_of!(AudioBufferListN<1>, m_number_buffers));
        assert_eq!(8, offset_of!(AudioBufferListN<1>, buffers));
    }

    #[test]
    fn audio_format_linear_pcm_encodes_lpcm() {
        assert_eq!(0x6c70636d, K_AUDIO_FORMAT_LINEAR_PCM);
    }

    #[test]
    fn build_output_asbd_stereo_48k_interleaved_float32() {
        let a = build_output_asbd(48_000.0, 2);
        assert_eq!(48_000.0, a.m_sample_rate);
        assert_eq!(K_AUDIO_FORMAT_LINEAR_PCM, a.m_format_id);
        assert_eq!(
            K_LINEAR_PCM_FORMAT_FLAG_IS_FLOAT | K_LINEAR_PCM_FORMAT_FLAG_IS_PACKED,
            a.m_format_flags
        );
        assert_eq!(8, a.m_bytes_per_packet);
        assert_eq!(1, a.m_frames_per_packet);
        assert_eq!(8, a.m_bytes_per_frame);
        assert_eq!(2, a.m_channels_per_frame);
        assert_eq!(32, a.m_bits_per_channel);
        assert_eq!(0, a.m_reserved);
    }

    #[test]
    fn build_output_asbd_mono_441k_interleaved_float32() {
        let a = build_output_asbd(44_100.0, 1);
        assert_eq!(44_100.0, a.m_sample_rate);
        assert_eq!(4, a.m_bytes_per_packet);
        assert_eq!(4, a.m_bytes_per_frame);
        assert_eq!(1, a.m_channels_per_frame);
    }

    #[test]
    fn build_input_asbd_non_interleaved_sets_stride_to_4_bytes_per_channel_plane() {
        let a = build_input_asbd(48_000.0, 2, true);
        assert_ne!(
            0,
            a.m_format_flags & K_LINEAR_PCM_FORMAT_FLAG_IS_NON_INTERLEAVED
        );
        assert_eq!(4, a.m_bytes_per_frame);
        assert_eq!(4, a.m_bytes_per_packet);
    }

    #[test]
    fn build_input_asbd_interleaved_omits_non_interleaved_flag() {
        let a = build_input_asbd(48_000.0, 2, false);
        assert_eq!(
            0,
            a.m_format_flags & K_LINEAR_PCM_FORMAT_FLAG_IS_NON_INTERLEAVED
        );
        assert_eq!(8, a.m_bytes_per_frame);
    }

    #[test]
    fn native_f32_predicates_require_packed_little_endian_layout() {
        let interleaved = build_input_asbd(48_000.0, 2, false);
        let planar = build_input_asbd(48_000.0, 2, true);
        assert!(is_native_f32_interleaved(interleaved));
        assert!(is_native_f32_planar(planar));
        let mut padded = interleaved;
        padded.m_format_flags &= !K_LINEAR_PCM_FORMAT_FLAG_IS_PACKED;
        assert!(!is_native_f32_interleaved(padded));
        let mut big_endian = planar;
        big_endian.m_format_flags |= K_LINEAR_PCM_FORMAT_FLAG_IS_BIG_ENDIAN;
        assert!(!is_native_f32_planar(big_endian));
    }

    #[test]
    fn output_frame_capacity_rounds_up_across_rate_ratios() {
        assert_eq!(1025, output_frame_capacity(1024, 48_000.0, 48_000.0));
        assert_eq!(1116, output_frame_capacity(1024, 44_100.0, 48_000.0));
        assert_eq!(512, output_frame_capacity(512, 0.0, 48_000.0));
    }

    #[test]
    fn converted_frame_count_computes_exact_output_length_without_safety_padding() {
        assert_eq!(1024, converted_frame_count(1024, 48_000.0, 48_000.0));
        assert_eq!(1115, converted_frame_count(1024, 44_100.0, 48_000.0));
    }

    #[test]
    fn convert_buffer_list_to_interleaved_f32_duplicates_mono_float32() {
        let samples = [0.25_f32, -0.5, 1.0];
        let list = AudioBufferListN {
            m_number_buffers: 1,
            buffers: [AudioBuffer::from_slice(1, &samples)],
        };
        let asbd = build_input_asbd(48_000.0, 1, false);
        let mut out = [0.0_f32; 6];
        let frames =
            convert_buffer_list_to_interleaved_f32(asbd, &list, 3, 48_000.0, 2, &mut out).unwrap();
        assert_eq!(3, frames);
        assert_eq!([0.25, 0.25, -0.5, -0.5, 1.0, 1.0], out);
    }

    #[test]
    fn convert_buffer_list_to_interleaved_f32_copies_native_stereo_float32() {
        let samples = [0.25_f32, -0.5, 1.0, -1.0, 0.125, -0.125];
        let list = AudioBufferListN {
            m_number_buffers: 1,
            buffers: [AudioBuffer::from_slice(2, &samples)],
        };
        let asbd = build_input_asbd(48_000.0, 2, false);
        let mut out = [0.0_f32; 6];
        let frames =
            convert_buffer_list_to_interleaved_f32(asbd, &list, 3, 48_000.0, 2, &mut out).unwrap();
        assert_eq!(3, frames);
        assert_eq!(samples, out);
    }

    #[test]
    fn convert_buffer_list_to_interleaved_f32_interleaves_planar_stereo_float32() {
        let left = [0.1_f32, 0.2, 0.3];
        let right = [-0.1_f32, -0.2, -0.3];
        let list = AudioBufferListN {
            m_number_buffers: 2,
            buffers: [
                AudioBuffer::from_slice(1, &left),
                AudioBuffer::from_slice(1, &right),
            ],
        };
        let asbd = build_input_asbd(48_000.0, 2, true);
        let mut out = [0.0_f32; 6];
        let frames =
            convert_buffer_list_to_interleaved_f32(asbd, &list, 3, 48_000.0, 2, &mut out).unwrap();
        assert_eq!(3, frames);
        assert_eq!([0.1, -0.1, 0.2, -0.2, 0.3, -0.3], out);
    }

    #[test]
    fn input_frame_count_for_buffer_list_handles_interleaved_and_planar_input() {
        let interleaved = [0.1_f32, -0.1, 0.2, -0.2, 0.3, -0.3];
        let interleaved_list = AudioBufferListN {
            m_number_buffers: 1,
            buffers: [AudioBuffer::from_slice(2, &interleaved)],
        };
        assert_eq!(
            3,
            input_frame_count_for_buffer_list(
                build_input_asbd(48_000.0, 2, false),
                &interleaved_list
            )
            .unwrap()
        );
        let left = [0.1_f32, 0.2, 0.3, 0.4];
        let right = [-0.1_f32, -0.2, -0.3, -0.4];
        let planar_list = AudioBufferListN {
            m_number_buffers: 2,
            buffers: [
                AudioBuffer::from_slice(1, &left),
                AudioBuffer::from_slice(1, &right),
            ],
        };
        assert_eq!(
            4,
            input_frame_count_for_buffer_list(build_input_asbd(48_000.0, 2, true), &planar_list)
                .unwrap()
        );
    }

    #[test]
    fn convert_buffer_list_to_interleaved_f32_normalizes_signed_int16() {
        let samples = [0_i16, 16_384, -32_768, 32_767];
        let list = AudioBufferListN {
            m_number_buffers: 1,
            buffers: [AudioBuffer::from_slice(2, &samples)],
        };
        let mut asbd = build_input_asbd(48_000.0, 2, false);
        asbd.m_format_flags =
            K_LINEAR_PCM_FORMAT_FLAG_IS_SIGNED_INTEGER | K_LINEAR_PCM_FORMAT_FLAG_IS_PACKED;
        asbd.m_bytes_per_packet = 4;
        asbd.m_bytes_per_frame = 4;
        asbd.m_bits_per_channel = 16;
        let mut out = [0.0_f32; 4];
        let frames =
            convert_buffer_list_to_interleaved_f32(asbd, &list, 2, 48_000.0, 2, &mut out).unwrap();
        assert_eq!(2, frames);
        assert!((out[0] - 0.0).abs() < 0.00001);
        assert!((out[1] - 0.5).abs() < 0.00001);
        assert!((out[2] - -1.0).abs() < 0.00001);
        assert!((out[3] - 0.9999695).abs() < 0.00001);
    }

    #[test]
    fn convert_buffer_list_to_interleaved_f32_handles_float64_input() {
        let samples = [-0.25_f64, 0.75];
        let list = AudioBufferListN {
            m_number_buffers: 1,
            buffers: [AudioBuffer::from_slice(1, &samples)],
        };
        let mut asbd = build_input_asbd(48_000.0, 1, false);
        asbd.m_bytes_per_packet = 8;
        asbd.m_bytes_per_frame = 8;
        asbd.m_bits_per_channel = 64;
        let mut out = [0.0_f32; 4];
        let frames =
            convert_buffer_list_to_interleaved_f32(asbd, &list, 2, 48_000.0, 2, &mut out).unwrap();
        assert_eq!(2, frames);
        assert_eq!([-0.25, -0.25, 0.75, 0.75], out);
    }

    #[test]
    fn convert_buffer_list_to_interleaved_f32_handles_padded_float32_fallback() {
        let mut bytes = [0_u8; 16];
        bytes[0..4].copy_from_slice(&0.5_f32.to_ne_bytes());
        bytes[8..12].copy_from_slice(&(-0.25_f32).to_ne_bytes());
        let list = AudioBufferListN {
            m_number_buffers: 1,
            buffers: [AudioBuffer::from_slice(1, &bytes)],
        };
        let mut asbd = build_input_asbd(48_000.0, 1, false);
        asbd.m_format_flags &= !K_LINEAR_PCM_FORMAT_FLAG_IS_PACKED;
        asbd.m_bytes_per_packet = 8;
        asbd.m_bytes_per_frame = 8;
        let mut out = [0.0_f32; 4];
        let frames =
            convert_buffer_list_to_interleaved_f32(asbd, &list, 2, 48_000.0, 2, &mut out).unwrap();
        assert_eq!(2, frames);
        assert_eq!([0.5, 0.5, -0.25, -0.25], out);
    }

    #[test]
    fn convert_buffer_list_to_interleaved_f32_linearly_resamples_to_target_rate() {
        let samples = [0.0_f32, 1.0];
        let list = AudioBufferListN {
            m_number_buffers: 1,
            buffers: [AudioBuffer::from_slice(1, &samples)],
        };
        let asbd = build_input_asbd(24_000.0, 1, false);
        let mut out = [0.0_f32; 8];
        let frames =
            convert_buffer_list_to_interleaved_f32(asbd, &list, 2, 48_000.0, 2, &mut out).unwrap();
        assert_eq!(4, frames);
        assert_eq!([0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.0, 1.0], out);
    }
}
