#!/usr/bin/env python3
"""Inspect the HBX0 module table without interpreting embedded bytecode."""

from __future__ import annotations

import dataclasses
import hashlib
import pathlib
import re
from collections.abc import Iterable

MAGIC = b"HBX0"
HEADER_SIZE = 4 + 32


class HbxFormatError(ValueError):
    """Raised when an HBX0 container is malformed or fails its checksum."""


@dataclasses.dataclass(frozen=True)
class HbxModule:
    resource: str
    namespace_form: str
    dependencies: tuple[str, ...]
    eager: bool


class _Reader:
    def __init__(self, data: bytes):
        self.data = data
        self.offset = 0

    def take(self, size: int, label: str) -> bytes:
        if size < 0 or self.offset + size > len(self.data):
            raise HbxFormatError(f"truncated {label}")
        start = self.offset
        self.offset += size
        return self.data[start : self.offset]

    def u32(self, label: str) -> int:
        return int.from_bytes(self.take(4, label), "little")

    def blob(self, label: str) -> bytes:
        return self.take(self.u32(f"{label} length"), label)

    def text(self, label: str) -> str:
        try:
            return self.blob(label).decode("utf-8")
        except UnicodeDecodeError as error:
            raise HbxFormatError(f"{label} is not UTF-8") from error

    def finished(self) -> bool:
        return self.offset == len(self.data)


def decode_module_table(data: bytes) -> list[HbxModule]:
    """Decode HBX0 descriptors while skipping each module's artifact bytes."""
    if len(data) < HEADER_SIZE or data[:4] != MAGIC:
        raise HbxFormatError("invalid HBX0 header")
    payload = data[HEADER_SIZE:]
    if hashlib.sha256(payload).digest() != data[4:HEADER_SIZE]:
        raise HbxFormatError("HBX0 checksum mismatch")

    reader = _Reader(payload)
    modules: list[HbxModule] = []
    resources: set[str] = set()
    for module_index in range(reader.u32("module count")):
        prefix = f"module {module_index}"
        resource = reader.text(f"{prefix} resource")
        if resource in resources:
            raise HbxFormatError(f"duplicate HBX0 module resource: {resource}")
        resources.add(resource)
        namespace_form = reader.text(f"{prefix} namespace form")
        reader.take(32, f"{prefix} source digest")
        dependencies = tuple(
            reader.text(f"{prefix} dependency {dependency_index}")
            for dependency_index in range(reader.u32(f"{prefix} dependency count"))
        )
        eager_value = reader.take(1, f"{prefix} eager flag")[0]
        if eager_value not in (0, 1):
            raise HbxFormatError(f"{prefix} has invalid eager flag {eager_value}")
        reader.blob(f"{prefix} bytecode artifact")
        modules.append(
            HbxModule(
                resource=resource,
                namespace_form=namespace_form,
                dependencies=dependencies,
                eager=bool(eager_value),
            )
        )

    if not reader.finished():
        raise HbxFormatError("trailing bytes in HBX0 container")
    return modules


def read_module_table(path: pathlib.Path) -> list[HbxModule]:
    return decode_module_table(path.read_bytes())


def _namespace_mentions(namespace_form: str, namespace: str) -> bool:
    token = re.compile(
        rf"(?<![A-Za-z0-9_.-]){re.escape(namespace)}(?![A-Za-z0-9_.-])"
    )
    return token.search(namespace_form) is not None


def retired_module_references(
    path: pathlib.Path, retired_namespaces: Iterable[str]
) -> list[str]:
    """Return retired references from HBX descriptors, never artifact bytes."""
    retired = tuple(dict.fromkeys(retired_namespaces))
    references: set[str] = set()
    for module in read_module_table(path):
        for namespace in retired:
            if module.resource == namespace:
                references.add(f"resource {namespace}")
                continue
            if namespace in module.dependencies:
                references.add(f"{module.resource} dependency {namespace}")
                continue
            if _namespace_mentions(module.namespace_form, namespace):
                references.add(f"{module.resource} namespace form {namespace}")
    return sorted(references)
