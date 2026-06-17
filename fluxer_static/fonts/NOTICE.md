# Font licenses

The font files in this directory are third-party font software. The CSS files
(`ibm-plex.css`, `bricolage.css`) are Fluxer-authored configuration for serving
those fonts.

## IBM Plex

IBM Plex is licensed under the SIL Open Font License 1.1, copied in
`LICENSE-IBM-PLEX.txt`. Its copyright line declares Reserved Font Name "Plex".

The served WOFF2 files are sourced from the latest per-family npm packages:

- **`@ibm/plex-sans@1.1.0`** (`IBMPlexSans/`)
- **`@ibm/plex-mono@2.5.0`** (`IBMPlexMono/`)
- **`@ibm/plex-sans-arabic@1.1.0`** (`IBMPlexSansArabic/`)
- **`@ibm/plex-sans-devanagari@1.1.0`** (`IBMPlexSansDevanagari/`)
- **`@ibm/plex-sans-hebrew@1.1.0`** (`IBMPlexSansHebrew/`)
- **`@ibm/plex-sans-thai@1.1.0`** (`IBMPlexSansThai/`)
- **`@ibm/plex-sans-thai-looped@1.1.0`** (`IBMPlexSansThaiLooped/`)
- **`@ibm/plex-sans-jp@3.0.0`** (`IBMPlexSansJP/`)
- **`@ibm/plex-sans-kr@1.1.0`** (`IBMPlexSansKR/`)
- **`@ibm/plex-sans-sc@1.1.0`** (`IBMPlexSansSC/`)
- **`@ibm/plex-sans-tc@1.1.1`** (`IBMPlexSansTC/`)

### Fluxer modifications

The upstream static WOFF2 files ship a staged `gasp` table
(`{8: 10, 16: 7 or 5, 65535: 15}`) that withholds symmetric smoothing (and, for
some scripts, grayscale) between 9 and 16 ppem. Browsers that honor `gasp`,
notably Firefox and Windows DirectWrite, then render the affected weights with
harsh, aliased edges at typical UI sizes, while macOS and Chrome look fine. There
is no CSS-level override for `gasp`, so the table is corrected in the font binary.

Each served WOFF2 in the families listed below has its `gasp` table normalized to
`{8: 10, 65535: 15}` (grid-fit plus grayscale plus symmetric smoothing from 9 ppem
up). Outlines, hand-hinting (`fpgm`, `prep`, `cvt`), glyph coverage, and vertical
metrics are unchanged.

Because the OFL reserves the name "Plex", a Modified Version may not present that
name to users. The modified font binaries are therefore renamed in their identity
name records (family, full, PostScript, and typographic-family fields):

- **IBM Plex Sans** to **Fluxer Sans**
- **IBM Plex Mono** to **Fluxer Mono**
- **IBM Plex Sans Arabic** to **Fluxer Sans Arabic**
- **IBM Plex Sans Devanagari** to **Fluxer Sans Devanagari**
- **IBM Plex Sans Hebrew** to **Fluxer Sans Hebrew**
- **IBM Plex Sans Thai** to **Fluxer Sans Thai**
- **IBM Plex Sans Thai Looped** to **Fluxer Sans Thai Looped**

The copyright, license, trademark, manufacturer, designer, and vendor name records
inside every font are preserved unchanged; only the `gasp` table and the identity
name records were altered. File and directory names keep their upstream form
(`IBMPlexSans/IBMPlexSans-Regular.woff2`), since OFL clause 3 concerns the font
name presented to users, not file paths.

### Backward-compatibility aliases

`ibm-plex.css` declares each modified family under both its new **Fluxer** name
(canonical) and its legacy **IBM Plex** name, with both `@font-face` entries
pointing at the same corrected WOFF2 files. This keeps any existing references to
the old names (cached app shells, saved user themes, external surfaces) working,
and they receive the same rendering fix. The redistributed font software itself
carries only the compliant Fluxer name; the legacy name exists solely as CSS
configuration, not as a font name baked into a distributed binary.

### Unmodified families

These keep the at-small-sizes behavior already fixed upstream (CJK ships
`{…, 65535: 15}` from 8 ppem, Korean ships no `gasp`), so they are served
unmodified and retain their original IBM Plex names:

- **IBM Plex Sans JP**, **IBM Plex Sans KR**, **IBM Plex Sans SC**, **IBM Plex Sans TC**

The `.otf` files in each family directory are the unmodified upstream originals and
are not referenced by any CSS; only the `.woff2` files are served.

## Bricolage Grotesque

`@fontsource/bricolage-grotesque@5.2.10` (`BricolageGrotesque/`), licensed under
the SIL Open Font License 1.1, copied in `LICENSE-BRICOLAGE-GROTESQUE.txt`. It has
no reserved font name, already ships a smooth `gasp` table (`{65535: 15}`), and is
served unmodified.
