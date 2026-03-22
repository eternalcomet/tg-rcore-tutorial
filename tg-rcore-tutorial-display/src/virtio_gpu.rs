use crate::{DisplayError, FrameBuffer, PixelFormat};
use core::{ptr::NonNull, slice};
use virtio_drivers::{Hal, MmioTransport, VirtIOGpu, VirtIOHeader};

const DMA_POOL_SIZE: usize = 16 * 1024 * 1024;
const PAGE_SIZE: usize = 4096;

#[repr(C, align(4096))]
struct DmaPool([u8; DMA_POOL_SIZE]);

static mut DMA_POOL: DmaPool = DmaPool([0; DMA_POOL_SIZE]);
static mut DMA_OFFSET: usize = 0;

struct VirtioHal;

impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> usize {
        let size = pages * PAGE_SIZE;
        // SAFETY: Single-core early boot environment; simple bump allocator.
        unsafe {
            let aligned = (DMA_OFFSET + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
            if aligned + size > DMA_POOL_SIZE {
                return 0;
            }
            DMA_OFFSET = aligned + size;
            core::ptr::addr_of_mut!(DMA_POOL.0) as usize + aligned
        }
    }

    fn dma_dealloc(_paddr: usize, _pages: usize) -> i32 {
        0
    }

    fn phys_to_virt(paddr: usize) -> usize {
        paddr
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        vaddr
    }
}

pub(crate) struct VirtioGpuDisplay {
    gpu: VirtIOGpu<'static, VirtioHal, MmioTransport>,
    width: usize,
    height: usize,
    fb_ptr: NonNull<u8>,
    fb_len: usize,
}

impl VirtioGpuDisplay {
    pub(crate) fn new(mmio_base: usize) -> Result<Self, DisplayError> {
        let transport = unsafe {
            MmioTransport::new(
                NonNull::new(mmio_base as *mut VirtIOHeader)
                    .ok_or(DisplayError::BadMmioTransport)?,
            )
        }
        .map_err(|_| DisplayError::BadMmioTransport)?;

        let mut gpu = VirtIOGpu::<'static, VirtioHal, _>::new(transport)
            .map_err(|_| DisplayError::BadGpuDevice)?;

        let (w, h) = gpu
            .resolution()
            .map_err(|_| DisplayError::GpuOperationFailed)?;
        let fb = gpu
            .setup_framebuffer()
            .map_err(|_| DisplayError::GpuOperationFailed)?;

        if w == 0 || h == 0 {
            return Err(DisplayError::InvalidFrameBuffer);
        }

        let fb_ptr = NonNull::new(fb.as_mut_ptr()).ok_or(DisplayError::InvalidFrameBuffer)?;
        let fb_len = fb.len();

        Ok(Self {
            gpu,
            width: w as usize,
            height: h as usize,
            fb_ptr,
            fb_len,
        })
    }

    pub(crate) fn framebuffer(&mut self) -> Result<FrameBuffer<'_>, DisplayError> {
        let width = self.width;
        let height = self.height;
        let stride = width * 4;
        let bytes_per_pixel = 4usize;
        let len = stride
            .checked_mul(height)
            .ok_or(DisplayError::InvalidFrameBuffer)?;

        if self.fb_len < len {
            return Err(DisplayError::InvalidFrameBuffer);
        }

        // SAFETY: fb_ptr/fb_len come from virtio-drivers setup_framebuffer and
        // remain valid as long as self.gpu owns frame_buffer_dma.
        let raw = unsafe { slice::from_raw_parts_mut(self.fb_ptr.as_ptr(), self.fb_len) };

        Ok(FrameBuffer {
            width,
            height,
            stride,
            bytes_per_pixel,
            format: PixelFormat::Argb8888,
            has_alpha: true,
            data: &mut raw[..len],
        })
    }

    pub(crate) fn flush(&mut self) -> Result<(), DisplayError> {
        self.gpu
            .flush()
            .map_err(|_| DisplayError::GpuOperationFailed)
    }
}
