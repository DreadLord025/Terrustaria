//! Section 2 (pointer 1): the world header + flags —
//! `WorldFile.SaveWorldHeader:1221` / `WorldFile.SaveWorldFlags:1238`.
//!
//! Phase 1 stub: both writers share section pointer 1, so `save` calls
//! `header::write` then this one. The header + flags writer phase replaces
//! this body (every field in C# order, port-owned fields from `World`, C#
//! defaults elsewhere).

use crate::binio::Writer;
use crate::world::World;

pub fn implemented() -> bool {
    false
}

pub fn write(_w: &mut Writer, _world: &World) {}
