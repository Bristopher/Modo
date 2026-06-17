# Bundled segmentation models

## selfie_segmenter_landscape.onnx

- **Source**: Google MediaPipe Selfie Segmenter (landscape), `selfie_segmenter_landscape.tflite`, downloaded from `https://storage.googleapis.com/mediapipe-models/image_segmenter/selfie_segmenter_landscape/float16/latest/selfie_segmenter_landscape.tflite` (sha256 `490e9ea734313e0de10fa0cd9e3c6133e36ea4db2b7a49bde9ef019f72796b8e`).
- **License**: Apache License 2.0, per the official model card ("Model Card MediaPipe Selfie Segmentation", Google, 2021; `https://storage.googleapis.com/mediapipe-assets/Model%20Card%20MediaPipe%20Selfie%20Segmentation.pdf`). This is the Apache-licensed Selfie model, not the ToS-restricted Google Meet model (`segm_full_v679.tflite`), which must never be shipped.
- **Conversion**: `tf2onnx` (`python -m tf2onnx.convert --tflite selfie_segmenter_landscape.tflite --output model.onnx --opset 13`), followed by graph surgery that replaces the single `TFL_Convolution2DTransposeBias` custom op with a standard `ConvTranspose` (weights transposed from `[out, kh, kw, in]` to `[in, out, kh, kw]`, strides 2x2, no padding) wrapped in NHWC/NCHW transposes, and bumps the default opset domain to 14 for `HardSwish`.
- **Verification**: output of the converted model matches the original tflite interpreter to a max abs diff of 8.7e-8 on random input. On a portrait test image the output is person confidence (1.0 on the subject, 0.0 in background corners), despite the output tensor name `segment_back`.
- **Signature**: input `input_1` `[1, 144, 256, 3]` f32 RGB scaled to 0..1, output `segment_back` `[1, 144, 256, 1]` f32 person confidence 0..1.
- **sha256**: `e8224061bba6031282bfd00cf23a2563fa11a1e28baae0a9052cef6b4e7f3321`
