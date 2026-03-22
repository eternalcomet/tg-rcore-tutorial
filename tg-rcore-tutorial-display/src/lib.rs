//! Reusable framebuffer display abstraction for tutorial kernels.

#![no_std]
#![cfg_attr(target_arch = "riscv64", deny(warnings, missing_docs))]
#![cfg_attr(not(target_arch = "riscv64"), allow(dead_code))]

#[cfg(target_arch = "riscv64")]
mod virtio_gpu;

/// QEMU virt default MMIO base for the first VirtIO device.
pub const DEFAULT_VIRTIO_MMIO_BASE: usize = 0x1000_1000;

/// Pixel storage format of framebuffer bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    /// 32-bit X8R8G8B8 (little-endian in memory).
    Xrgb8888,
    /// 32-bit A8R8G8B8 (little-endian in memory).
    Argb8888,
    /// 32-bit B8G8R8X8 (little-endian in memory).
    Bgrx8888,
    /// 32-bit B8G8R8A8 (little-endian in memory).
    Bgra8888,
    /// Unknown/unspecified format.
    Unknown,
}

/// Display-related error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayError {
    /// Failed to create VirtIO MMIO transport.
    BadMmioTransport,
    /// Failed to initialize VirtIO GPU device.
    BadGpuDevice,
    /// GPU operation failed.
    GpuOperationFailed,
    /// Framebuffer geometry is invalid.
    InvalidFrameBuffer,
    /// Unsupported architecture placeholder.
    UnsupportedArch,
}

/// Mutable framebuffer view.
pub struct FrameBuffer<'a> {
    /// Pixel width.
    pub width: usize,
    /// Pixel height.
    pub height: usize,
    /// Bytes per scanline.
    pub stride: usize,
    /// Bytes per pixel.
    pub bytes_per_pixel: usize,
    /// Storage format.
    pub format: PixelFormat,
    /// Whether alpha channel is interpreted by scanout.
    pub has_alpha: bool,
    data: &'a mut [u8],
}

impl<'a> FrameBuffer<'a> {
    /// Returns raw framebuffer bytes.
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.data
    }

    /// Returns byte offset of pixel `(x, y)`.
    #[inline]
    pub fn byte_offset(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(y * self.stride + x * self.bytes_per_pixel)
    }
}

/// Display device abstraction.
pub struct Display {
    #[cfg(target_arch = "riscv64")]
    inner: virtio_gpu::VirtioGpuDisplay,
}

impl Display {
    /// Creates a display from a VirtIO-GPU MMIO base address.
    #[cfg(target_arch = "riscv64")]
    pub fn new_virtio_gpu(mmio_base: usize) -> Result<Self, DisplayError> {
        let inner = virtio_gpu::VirtioGpuDisplay::new(mmio_base)?;
        Ok(Self { inner })
    }

    /// Placeholder for non-riscv targets.
    #[cfg(not(target_arch = "riscv64"))]
    pub fn new_virtio_gpu(_mmio_base: usize) -> Result<Self, DisplayError> {
        Err(DisplayError::UnsupportedArch)
    }

    /// Returns framebuffer metadata with mutable byte access.
    #[cfg(target_arch = "riscv64")]
    pub fn framebuffer(&mut self) -> Result<FrameBuffer<'_>, DisplayError> {
        self.inner.framebuffer()
    }

    /// Placeholder for non-riscv targets.
    #[cfg(not(target_arch = "riscv64"))]
    pub fn framebuffer(&mut self) -> Result<FrameBuffer<'_>, DisplayError> {
        let _ = self;
        Err(DisplayError::UnsupportedArch)
    }

    /// Flushes current framebuffer content to screen.
    #[cfg(target_arch = "riscv64")]
    pub fn flush(&mut self) -> Result<(), DisplayError> {
        self.inner.flush()
    }

    /// Placeholder for non-riscv targets.
    #[cfg(not(target_arch = "riscv64"))]
    pub fn flush(&mut self) -> Result<(), DisplayError> {
        let _ = self;
        Err(DisplayError::UnsupportedArch)
    }
}
