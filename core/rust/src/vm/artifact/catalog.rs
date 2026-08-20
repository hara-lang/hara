use super::{read_schema_type, write_schema_type, Reader, Writer};
use crate::kernel::schema::SchemaType;
use crate::vm::Program;
use std::cmp::Ordering;

const SECTION_MARKER: &[u8; 4] = b"SCAT";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaCoordinate {
    pub id: String,
    pub version: u32,
    pub hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaCatalogEntry {
    pub coordinate: SchemaCoordinate,
    pub schema: SchemaType,
    pub dependencies: Vec<SchemaCoordinate>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecodedArtifact {
    pub program: Program,
    pub schema_catalog: Vec<SchemaCatalogEntry>,
}

fn coordinate_order(left: &SchemaCoordinate, right: &SchemaCoordinate) -> Ordering {
    left.id
        .as_bytes()
        .cmp(right.id.as_bytes())
        .then_with(|| left.version.cmp(&right.version))
        .then_with(|| left.hash.as_bytes().cmp(right.hash.as_bytes()))
}

fn entry_order(left: &SchemaCatalogEntry, right: &SchemaCatalogEntry) -> Ordering {
    coordinate_order(&left.coordinate, &right.coordinate)
}

fn valid_identity(value: &str) -> bool {
    let Some((namespace, name)) = value.split_once('/') else {
        return false;
    };
    !namespace.is_empty() && !name.is_empty() && !name.contains('/') && !value.starts_with(':')
}

fn valid_hash(value: &str) -> bool {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return false;
    };
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_coordinate(value: &SchemaCoordinate) -> Result<(), String> {
    if !valid_identity(&value.id) {
        return Err("bytecode schema catalog contains invalid schema identity".into());
    }
    if !valid_hash(&value.hash) {
        return Err("bytecode schema catalog contains invalid schema hash".into());
    }
    Ok(())
}

fn canonical_entries(entries: &[SchemaCatalogEntry]) -> Result<Vec<SchemaCatalogEntry>, String> {
    let mut output = entries.to_vec();
    for entry in &mut output {
        validate_coordinate(&entry.coordinate)?;
        for dependency in &entry.dependencies {
            validate_coordinate(dependency)?;
        }
        entry.dependencies.sort_by(coordinate_order);
        if entry
            .dependencies
            .windows(2)
            .any(|window| window[0] == window[1])
        {
            return Err("bytecode schema catalog contains duplicate dependency coordinate".into());
        }
    }
    output.sort_by(entry_order);
    if output
        .windows(2)
        .any(|window| window[0].coordinate == window[1].coordinate)
    {
        return Err("bytecode schema catalog contains duplicate schema coordinate".into());
    }
    Ok(output)
}

fn write_coordinate(out: &mut Writer, value: &SchemaCoordinate) -> Result<(), String> {
    out.string(&value.id)?;
    out.u32(value.version);
    out.string(&value.hash)
}

fn read_coordinate(reader: &mut Reader<'_>) -> Result<SchemaCoordinate, String> {
    let value = SchemaCoordinate {
        id: reader.string()?,
        version: reader.u32()?,
        hash: reader.string()?,
    };
    validate_coordinate(&value)?;
    Ok(value)
}

pub(super) fn write_section(
    out: &mut Writer,
    entries: &[SchemaCatalogEntry],
) -> Result<(), String> {
    if entries.is_empty() {
        return Ok(());
    }
    let entries = canonical_entries(entries)?;
    out.bytes.extend_from_slice(SECTION_MARKER);
    out.len(entries.len())?;
    for entry in &entries {
        write_coordinate(out, &entry.coordinate)?;
        write_schema_type(out, &entry.schema)?;
        out.len(entry.dependencies.len())?;
        for dependency in &entry.dependencies {
            write_coordinate(out, dependency)?;
        }
    }
    Ok(())
}

pub(super) fn read_section(reader: &mut Reader<'_>) -> Result<Vec<SchemaCatalogEntry>, String> {
    if reader.cursor == reader.bytes.len() {
        return Ok(Vec::new());
    }
    if reader.take(SECTION_MARKER.len())? != SECTION_MARKER {
        return Err("bytecode artifact contains unknown trailing section".into());
    }
    let entries = reader.many(|reader| {
        let coordinate = read_coordinate(reader)?;
        let schema = read_schema_type(reader)?;
        let dependencies = reader.many(read_coordinate)?;
        if dependencies
            .windows(2)
            .any(|window| coordinate_order(&window[0], &window[1]) != Ordering::Less)
        {
            return Err("bytecode schema dependencies are not canonically ordered".into());
        }
        Ok(SchemaCatalogEntry {
            coordinate,
            schema,
            dependencies,
        })
    })?;
    if entries
        .windows(2)
        .any(|window| entry_order(&window[0], &window[1]) != Ordering::Less)
    {
        return Err("bytecode schema catalog is not canonically ordered".into());
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coordinate(id: &str, version: u32, digit: char) -> SchemaCoordinate {
        SchemaCoordinate {
            id: id.into(),
            version,
            hash: format!("sha256:{}", digit.to_string().repeat(64)),
        }
    }

    #[test]
    fn canonicalizes_entries_and_dependencies() {
        let entries = vec![
            SchemaCatalogEntry {
                coordinate: coordinate("demo/z", 2, 'b'),
                schema: SchemaType::Primitive("int".into()),
                dependencies: vec![coordinate("demo/b", 1, 'd'), coordinate("demo/a", 1, 'c')],
            },
            SchemaCatalogEntry {
                coordinate: coordinate("demo/a", 1, 'a'),
                schema: SchemaType::Primitive("str".into()),
                dependencies: vec![],
            },
        ];
        let canonical = canonical_entries(&entries).unwrap();
        assert_eq!(canonical[0].coordinate.id, "demo/a");
        assert_eq!(canonical[1].dependencies[0].id, "demo/a");
        assert_eq!(canonical[1].dependencies[1].id, "demo/b");
    }
}
