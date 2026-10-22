#!/usr/bin/env python3
"""Execute NRO initialization in an AArch64 sandbox with modeled loader calls.

Requires unicorn==2.1.4. This checks startup control flow, not the game itself.
"""

import argparse
from pathlib import Path
import re
import struct

from unicorn import Uc, UC_ARCH_ARM64, UC_MODE_ARM, UC_HOOK_CODE
from unicorn import arm64_const as registers
from nro_imports import required_imports
from verify_nro import verify_nro


ROOT = Path(__file__).resolve().parent.parent
BASE, GAME, STUB, STACK = (0x100000000, 0x200000000, 0x300000000, 0x400000000)
STOP = STUB + 0xF000


def metadata(data: bytes) -> tuple[list[tuple], list[tuple]]:
  # The bounded structure/import verifier has already accepted this image.
  mod, = struct.unpack_from("<I", data, 4)
  relative, = struct.unpack_from("<i", data, mod + 4)
  cursor = mod + relative
  tags = {}
  while True:
    tag, value = struct.unpack_from("<QQ", data, cursor)
    cursor += 16
    if not tag:
      break
    tags[tag] = value
  count, = struct.unpack_from("<I", data, tags[4] + 4)
  symbols = []
  for index in range(count):
    name, _, _, section, value, _ = struct.unpack_from("<IBBHQQ", data, tags[6] + index * 24)
    start = tags[5] + name
    symbols.append((data[start:data.index(0, start)].decode(), section, value))
  relocations = []
  for table, size in ((7, 8), (23, 2)):
    if table in tags:
      for cursor in range(tags[table], tags[table] + tags[size], 24):
        relocations.append(struct.unpack_from("<QQq", data, cursor))
  return symbols, relocations


def smoke(data: bytes, case: str) -> int:
  profile = (ROOT / "src/profile.rs").read_text(encoding="utf-8")

  def offset(name: str) -> int:
    return int(re.search(rf"pub const {name}: usize = (0x[0-9A-Fa-f]+)", profile).group(1), 16)

  signatures = [(int(address, 16), b"".join(
    struct.pack("<I", int(word, 16)) for word in re.findall(r"0x[0-9A-Fa-f]+", words)
  )) for address, words in re.findall(r"\((0x[0-9A-Fa-f]+),\s*\[([^\]]+)\]\)", profile)]
  id_values = re.search(r"pub const BUILD_ID:.*?=\s*\[(.*?)\];", profile, re.S).group(1)
  build_id = bytes(int(word, 16) for word in re.findall(r"0x[0-9A-Fa-f]+", id_values))
  expected_hooks = {offset(name) for name in (
    "ON_SELECT", "ON_DECIDE", "ON_YES", "ON_CANCEL", "DESTROY_MENU",
    "EXECUTE_MERGE", "UNLOAD", "MELD_ONE", "YES_NO_DIALOG"
  )}
  symbols, relocations = metadata(data)
  main = next(value for name, section, value in symbols if name == "main" and section)
  cpu = Uc(UC_ARCH_ARM64, UC_MODE_ARM)
  image_size = (len(data) + struct.unpack_from("<I", data, 0x38)[0] + 0xFFF) & ~0xFFF
  cpu.mem_map(BASE, image_size)
  cpu.mem_write(BASE, data)
  cpu.mem_map(STUB, 0x10000)
  cpu.mem_map(STACK, 0x20000)
  pages = set()

  def write_game(address: int, value: bytes) -> None:
    for page in range(address & ~0xFFF, (address + len(value) + 0xFFF) & ~0xFFF, 0x1000):
      if page not in pages:
        cpu.mem_map(page, 0x1000)
        pages.add(page)
    cpu.mem_write(address, value)

  for address, words in signatures:
    write_game(GAME + address, words)
  write_game(GAME + offset("BUILD_ID_OFFSET"), build_id if case != "wrong_build_id" else bytes(16))
  if case == "wrong_instruction":
    write_game(GAME + signatures[0][0], bytes(4))
  stubs = {name: STUB + index * 16 for index, name in enumerate(sorted(required_imports(data)))}
  names = {address: name for name, address in stubs.items()}
  for target, info, addend in relocations:
    index, kind = info >> 32, info & 0xFFFFFFFF
    if kind == 1027:  # R_AARCH64_RELATIVE
      value = BASE + addend
    elif kind in (257, 1025, 1026):  # ABS64, GLOB_DAT, JUMP_SLOT
      name, section, value = symbols[index]
      if not section:
        value = stubs[name]
      elif section != 0xFFF1:
        value += BASE
      value += addend
    else:
      raise ValueError(f"unsupported startup relocation {kind}")
    cpu.mem_write(BASE + target, struct.pack("<Q", value))
  hooks = []

  def native_call(cpu: Uc, pc: int, size: int, user: object) -> None:
    if pc == STOP:
      cpu.emu_stop()
      return
    if pc not in names:
      return
    name = names[pc]
    argument = cpu.reg_read(registers.UC_ARM64_REG_X0)
    if name == "getRegionAddress":
      delta = {0: 0, 1: offset("RODATA_OFFSET"), 2: 0x5D7C000}[argument]
      if case == "wrong_segment_layout" and argument == 1:
        delta += 0x1000
      cpu.reg_write(registers.UC_ARM64_REG_X0, GAME + delta)
    elif name == "_ZN2nn2oe17GetDisplayVersionEPNS0_14DisplayVersionE":
      cpu.mem_write(argument, b"1.3.0\0" if case == "wrong_version" else b"2.0.0\0")
    elif name == "A64HookFunction":
      replacement = cpu.reg_read(registers.UC_ARM64_REG_X1)
      if not BASE <= replacement < BASE + len(data):
        raise AssertionError("hook replacement is outside the plugin")
      hooks.append(argument - GAME)
      cpu.mem_write(cpu.reg_read(registers.UC_ARM64_REG_X2), struct.pack("<Q", argument))
    else:
      raise AssertionError(f"unexpected initialization import: {name}")
    cpu.reg_write(registers.UC_ARM64_REG_PC, cpu.reg_read(registers.UC_ARM64_REG_X30))

  cpu.hook_add(UC_HOOK_CODE, native_call)
  cpu.reg_write(registers.UC_ARM64_REG_SP, STACK + 0x20000 - 0x20)
  cpu.reg_write(registers.UC_ARM64_REG_X30, STOP)
  cpu.emu_start(BASE + main, STOP + 4, count=100_000)
  if cpu.reg_read(registers.UC_ARM64_REG_PC) != STOP:
    raise AssertionError(f"{case}: initialization did not return")
  expected = expected_hooks if case == "supported" else set()
  if set(hooks) != expected or len(hooks) != len(expected):
    raise AssertionError(f"{case}: incorrect hook registration: {hooks}")
  return len(hooks)


def main() -> None:
  parser = argparse.ArgumentParser(description=__doc__)
  parser.add_argument("nro", type=Path)
  args = parser.parse_args()
  verify_nro(args.nro)
  for case in ("supported", "wrong_version", "wrong_build_id", "wrong_instruction", "wrong_segment_layout"):
    count = smoke(args.nro.read_bytes(), case)
    print(f"{case}: initialization returned normally; {count} hooks; no unavailable API call")


if __name__ == "__main__":
  main()
