//! `EffectReader`: extracts the compiled effect's pass names from a `.xnb` Effect
//! asset (M1.4-BG-18 phase 2). Scope is deliberately narrow — names only, no HLSL
//! translation: the WGSL filter passes are hand-written from the pass semantics
//! (plan §Phase 2, "только имена пассов, не трансляция HLSL").

use std::path::Path;

use anyhow::{Result, bail};

use crate::xnb::{ByteReader, XnbFile};

/// Loads an Effect `.xnb` and returns its pass names in blob order.
pub fn load_effect_pass_names(path: impl AsRef<Path>) -> Result<Vec<String>> {
    let raw = std::fs::read(path.as_ref())?;
    let xnb = XnbFile::read(&raw)?;
    if !xnb.asset_reader()?.contains("EffectReader") {
        bail!("{}: not an Effect asset", path.as_ref().display());
    }
    pass_names_from_payload(&xnb.payload)
}

/// Reads one `Effect` object (`EffectReader.Read`): an int32 blob size followed by
/// the raw D3DX effect bytes. The D3DX object table is not parsed; the pass names
/// are the NUL-terminated ASCII identifiers embedded in the blob — the probe behind
/// this (phase-2 discovery) showed every `ScreenShader.fx` technique/pass name
/// appearing verbatim as one such run.
pub fn pass_names_from_payload(payload: &[u8]) -> Result<Vec<String>> {
    let mut br = ByteReader::new(payload);
    let size = br.read_i32()?;
    if size < 0 || size as usize > br.remaining() {
        bail!(
            "bad effect blob size {size} (payload {} bytes)",
            payload.len()
        );
    }
    let blob = br.read_bytes(size as usize)?;
    Ok(pass_names_from_blob(blob))
}

/// ASCII-run scan of the effect blob, keeping pass-name-shaped identifiers.
/// `ScreenShader.fx` pass names all start with `Filter`; everything else in the
/// blob is either a `u*` parameter name, an HLSL compiler string, or one of the
/// non-scene passes (`Default`, `OverlayGlow`, `MonoFix`, `Technique1`,
/// `WaterDebugDraw` — a `GameShaders.Misc` pass, `DyeInitializer.cs:427`).
fn pass_names_from_blob(blob: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    let mut start: Option<usize> = None;
    let mut run = 0usize;
    for (i, b) in blob.iter().enumerate() {
        let printable = (0x20..=0x7e).contains(b);
        if printable {
            if run == 0 {
                start = Some(i);
            }
            run += 1;
        } else {
            if run >= 4 {
                if let Some(s) = start.and_then(|s| std::str::from_utf8(&blob[s..i]).ok()) {
                    if is_pass_name(s) {
                        names.push(s.to_owned());
                    }
                }
            }
            run = 0;
        }
    }
    names
}

/// Pass-name shape: `Filter` + identifier characters (`FilterGraveyard`,
/// `FilterTest2`, …). Keeps the scan honest against random ASCII runs.
fn is_pass_name(s: &str) -> bool {
    s.len() > "Filter".len()
        && s.starts_with("Filter")
        && s["Filter".len()..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DW-2.4: the pass names extracted from the retail `ScreenShader.xnb` carry
    /// every pass `ScreenEffectInitializer.Load` registers
    /// (`ScreenEffectInitializer.cs:14-48`). The XNB set is a strict superset: the
    /// extra `FilterColor` pass exists in the effect but no initializer row uses it.
    #[test]
    fn test_DW_2_4_xnb_pass_names_match_initializer() {
        // The 22 unique pass names handed to `new ScreenShaderData(..)` /
        // `MoonLordScreenShaderData(..)` / etc. in `ScreenEffectInitializer.cs:14-48`.
        const INITIALIZER_PASSES: [&str; 22] = [
            "FilterTower",
            "FilterMiniTower",
            "FilterMoonLord",
            "FilterMoonLordShake",
            "FilterGraveyard",
            "FilterInvert",
            "FilterBloodMoon",
            "FilterSepia",
            "FilterNoir",
            "FilterSandstormForeground",
            "FilterSandstormBackground",
            "FilterBlizzardForeground",
            "FilterBlizzardBackground",
            "FilterHeatDistortion",
            "FilterWaterDistortion",
            "FilterCrystalDestructionColor",
            "FilterCrystalDestructionVortex",
            "FilterCrystalWin",
            "FilterTest",
            "FilterTest2",
            "FilterTest3",
            "FilterCRT",
        ];
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/Content/ScreenShader.xnb"
        );
        let Ok(names) = load_effect_pass_names(path) else {
            eprintln!("ScreenShader.xnb not present; skipping");
            return;
        };
        for pass in INITIALIZER_PASSES {
            assert!(
                names.iter().any(|n| n == pass),
                "pass {pass} missing from ScreenShader.xnb (found: {names:?})"
            );
        }
        // The one XNB-only pass (documented above).
        assert!(names.contains(&"FilterColor".to_owned()), "{names:?}");
        assert_eq!(names.len(), 23, "{names:?}");
    }

    /// Pure scan on a synthetic payload: `Filter*` identifiers are kept in order,
    /// `u*` parameters / other pass names / short runs / non-ASCII are not.
    #[test]
    fn test_pass_names_from_payload_pure() {
        // Correct "size" i32, then the blob: parameter, two passes, a stray short
        // "Fil" run and a non-identifier "Filter-Type".
        let blob: &[&[u8]] = &[
            b"uColor\0FilterGraveyard\0Default\0",
            b"FilterTower\0FilterTest2\0Fil\0",
            b"Filter-Type\0Microsoft HLSL\0",
        ];
        let size = blob.iter().map(|b| b.len()).sum::<usize>() as i32;
        let mut payload: Vec<u8> = size.to_le_bytes().to_vec();
        for b in blob {
            payload.extend_from_slice(b);
        }
        payload.extend_from_slice(b"FilterTower\0FilterTest2\0Fil\0");
        payload.extend_from_slice(b"Filter-Type\0Microsoft HLSL\0");
        let names = pass_names_from_payload(&payload).unwrap();
        assert_eq!(names, ["FilterGraveyard", "FilterTower", "FilterTest2"]);
    }

    /// Malformed blobs are errors, not garbage lists.
    #[test]
    fn test_pass_names_from_payload_rejects_bad_size() {
        let payload = 0xFFFFi32.to_le_bytes().to_vec(); // size beyond the payload
        assert!(pass_names_from_payload(&payload).is_err());
        let payload = (-1i32).to_le_bytes().to_vec();
        assert!(pass_names_from_payload(&payload).is_err());
    }
}
