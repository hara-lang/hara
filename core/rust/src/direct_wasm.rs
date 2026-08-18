//! Target-neutral discovery for the deliberately small direct `core.v1` ABI.

use crate::extension::ExtensionExport;

pub(crate) fn exports(bytes: &[u8]) -> Result<Vec<(String, ExtensionExport)>, String> {
    if bytes.get(..8) != Some(b"\0asm\x01\0\0\0") {
        return Err("native/module-invalid: invalid WebAssembly header".into());
    }
    let mut cursor = 8;
    let mut types = Vec::new();
    let mut functions = Vec::new();
    let mut exported = Vec::new();
    while cursor < bytes.len() {
        let id = byte(bytes, &mut cursor)?;
        let size = unsigned(bytes, &mut cursor)? as usize;
        let end = cursor
            .checked_add(size)
            .filter(|end| *end <= bytes.len())
            .ok_or("native/module-invalid: section exceeds module")?;
        let section = &bytes[cursor..end];
        cursor = end;
        let mut at = 0;
        match id {
            1 => {
                for _ in 0..unsigned(section, &mut at)? {
                    if byte(section, &mut at)? != 0x60 {
                        return Err("native/abi-type-unsupported: non-function type".into());
                    }
                    let arguments = value_types(section, &mut at)?;
                    let results = value_types(section, &mut at)?;
                    if results.len() > 1 {
                        return Err("native/abi-type-unsupported: multiple results".into());
                    }
                    types.push((arguments, results.into_iter().next().unwrap_or("void")));
                }
            }
            2 if unsigned(section, &mut at)? != 0 => {
                return Err("native/module-import-denied: direct WASM must be import-free".into())
            }
            3 => {
                for _ in 0..unsigned(section, &mut at)? {
                    functions.push(unsigned(section, &mut at)? as usize);
                }
            }
            5 => validate_memories(section, &mut at)?,
            7 => {
                for _ in 0..unsigned(section, &mut at)? {
                    let name = name(section, &mut at)?;
                    let kind = byte(section, &mut at)?;
                    let index = unsigned(section, &mut at)? as usize;
                    if kind == 0 {
                        exported.push((name, index));
                    }
                }
            }
            _ => {}
        }
        if matches!(id, 1 | 2 | 3 | 5 | 7) && at != section.len() {
            return Err(format!(
                "native/module-invalid: trailing bytes in section {id}"
            ));
        }
    }
    exported
        .into_iter()
        .map(|(name, index)| {
            let type_index = *functions
                .get(index)
                .ok_or_else(|| format!("native/module-invalid: bad function export {name}"))?;
            let (arguments, returns) = types
                .get(type_index)
                .ok_or_else(|| format!("native/module-invalid: bad type for export {name}"))?;
            Ok((
                name,
                ExtensionExport {
                    arguments: arguments.iter().map(|value| (*value).to_owned()).collect(),
                    returns: (*returns).to_owned(),
                    asynchronous: false,
                },
            ))
        })
        .collect()
}

fn validate_memories(bytes: &[u8], at: &mut usize) -> Result<(), String> {
    let count = unsigned(bytes, at)?;
    if count > 1 {
        return Err("native/resource-limit: at most one memory is allowed".into());
    }
    for _ in 0..count {
        let flags = unsigned(bytes, at)?;
        if flags > 1 {
            return Err("native/resource-limit: shared or 64-bit memories are unsupported".into());
        }
        let minimum = unsigned(bytes, at)?;
        let maximum = if flags & 1 != 0 {
            Some(unsigned(bytes, at)?)
        } else {
            None
        };
        if minimum > 1024 || maximum.is_none() || maximum.is_some_and(|value| value > 1024) {
            return Err("native/resource-limit: memory must be bounded to 64 MiB".into());
        }
    }
    Ok(())
}

fn value_types<'a>(bytes: &'a [u8], at: &mut usize) -> Result<Vec<&'static str>, String> {
    (0..unsigned(bytes, at)?)
        .map(|_| match byte(bytes, at)? {
            0x7f => Ok("i32"),
            0x7e => Ok("i64"),
            0x7d => Ok("f32"),
            0x7c => Ok("f64"),
            value => Err(format!("native/abi-type-unsupported: 0x{value:02x}")),
        })
        .collect()
}

fn name(bytes: &[u8], at: &mut usize) -> Result<String, String> {
    let size = unsigned(bytes, at)? as usize;
    let end = at
        .checked_add(size)
        .filter(|end| *end <= bytes.len())
        .ok_or("native/module-invalid: name exceeds section")?;
    let value = std::str::from_utf8(&bytes[*at..end])
        .map_err(|_| "native/module-invalid: export name is not UTF-8")?
        .to_owned();
    *at = end;
    Ok(value)
}

fn byte(bytes: &[u8], at: &mut usize) -> Result<u8, String> {
    let value = *bytes
        .get(*at)
        .ok_or("native/module-invalid: unexpected end of module")?;
    *at += 1;
    Ok(value)
}

fn unsigned(bytes: &[u8], at: &mut usize) -> Result<u32, String> {
    let mut value = 0u32;
    for shift in (0..35).step_by(7) {
        let byte = byte(bytes, at)?;
        if shift == 28 && byte & 0xf0 != 0 {
            return Err("native/module-invalid: integer overflow".into());
        }
        value |= u32::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("native/module-invalid: invalid integer".into())
}

#[cfg(test)]
mod tests {
    use super::exports;

    #[test]
    fn discovers_scalar_exports_without_a_host_engine() {
        let add = b"\0asm\x01\0\0\0\x01\x07\x01\x60\x02\x7e\x7e\x01\x7e\x03\x02\x01\0\x07\x07\x01\x03add\0\0\x0a\x09\x01\x07\0\x20\0\x20\x01\x7c\x0b";
        let found = exports(add).unwrap();
        assert_eq!(found[0].0, "add");
        assert_eq!(found[0].1.arguments, ["i64", "i64"]);
        assert_eq!(found[0].1.returns, "i64");
    }
}
