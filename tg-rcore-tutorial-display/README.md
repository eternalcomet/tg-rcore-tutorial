# tg-rcore-tutorial-display

Reusable no-std display crate for tg-rcore-tutorial kernels.

This crate provides a small and extensible framebuffer-oriented API based on
VirtIO-GPU MMIO on QEMU `virt`.

## Design goals

- Keep display initialization code reusable across chapter kernels.
- Expose framebuffer metadata and mutable pixel bytes to kernel code.
- Keep rendering policy in chapter code (for teaching and experiments).

## Public API (overview)

- `Display::new_virtio_gpu(mmio_base)`
- `Display::framebuffer() -> FrameBuffer`
- `Display::flush()`
- `FrameBuffer` fields and helpers:
  - `width`, `height`, `stride`, `bytes_per_pixel`, `format`
  - `as_bytes_mut()`
  - `byte_offset(x, y)`

## Typical usage

```ignore
let mut display = tg_display::Display::new_virtio_gpu(0x1000_1000)?;
{
    let mut fb = display.framebuffer();
    let pixels = fb.as_bytes_mut();
    // render into pixels
}
display.flush()?;
```

## Notes

- The crate intentionally does not provide high-level drawing primitives.
- Rendering algorithms/images should stay in chapter code (e.g. ch1 tangram).
