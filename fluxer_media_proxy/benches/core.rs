// SPDX-License-Identifier: AGPL-3.0-or-later

use criterion::{Criterion, criterion_group, criterion_main};
use fluxer_media_proxy::{aws_sigv4, mime, range, signing, thumbhash};

fn bench_range(c: &mut Criterion) {
    c.bench_function("range_parse_explicit", |b| {
        b.iter(|| range::parse_range(Some("bytes=1024-65535"), 10 * 1024 * 1024))
    });
}

fn bench_signing(c: &mut Criterion) {
    c.bench_function("external_signature", |b| {
        b.iter(|| {
            signing::create_signature("v2/aHR0cHM6Ly9leGFtcGxlLmNvbS9pbWFnZS5wbmc", b"secret")
        })
    });
    c.bench_function("aws_sigv4_get", |b| {
        b.iter(|| {
            let mut opts = aws_sigv4::Options::new(
                "GET",
                "https://examplebucket.s3.amazonaws.com/test.txt",
                "us-east-1",
                "AKIAIOSFODNN7EXAMPLE",
                "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            );
            opts.timestamp = Some(aws_sigv4::format_timestamp(2013, 5, 24, 0, 0, 0));
            aws_sigv4::sign(opts).unwrap()
        })
    });
}

fn bench_mime(c: &mut Criterion) {
    let png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x20\x00\x00\x00\x20\x08\x06";
    c.bench_function("mime_sniff_png", |b| b.iter(|| mime::sniff(png)));
}

fn bench_thumbhash(c: &mut Criterion) {
    let mut pixels = vec![0u8; 64 * 64 * 4];
    for (i, px) in pixels.chunks_exact_mut(4).enumerate() {
        px[0] = (i % 64) as u8;
        px[1] = (i / 64) as u8;
        px[2] = 128;
        px[3] = 255;
    }
    c.bench_function("thumbhash_64_rgba", |b| {
        b.iter(|| thumbhash::encode_rgba(&pixels, 64, 64).unwrap())
    });
}

criterion_group!(
    benches,
    bench_range,
    bench_signing,
    bench_mime,
    bench_thumbhash
);
criterion_main!(benches);
