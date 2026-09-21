use alloc::fmt::Write;
use core::ffi::{c_char, c_void};
use core::ptr::{NonNull, null_mut};
use core::sync::atomic::{AtomicBool, Ordering};

use ft_wrapper::{
    FLANTERM_FB_ROTATE_0,
    MALLOC_ALIGN,
    flanterm_context,
    flanterm_fb_init,
    flanterm_write,
};
use spin::LazyLock;

use crate::cpu::multiprocessor::spin::mutex::Mutex;
use crate::environment::boot_protocol::limine::FRAMEBUFFER_REQUEST;
use crate::log::chars::{FONT_HEIGHT, FONT_WIDTH};

/// Handle to the Flanterm terminal that kernel log output is written to.
///
/// Flanterm never exposes the layout of its context, so this holds the pointer the library handed
/// back rather than an owned value.
///
/// # Line endings
///
/// The terminal keeps the two halves of a line break separate, the way the teletypes this
/// vocabulary was named for did: a line feed (`\n`) drops the cursor one row and a carriage return
/// (`\r`) moves it back to column zero. Neither implies the other, so a bare `\n` leaves the
/// cursor in the column it was already in and the following line starts there.
///
/// Every line break written to this terminal is therefore `\r\n`. The [`logln!`](crate::logln)
/// macro appends one for its callers; anything that formats multiple lines of its own — a
/// [`Display`](core::fmt::Display) impl rendering a table or a tree, say — has to write `\r\n`
/// itself rather than reaching for [`writeln!`], which can only ever append a bare `\n`.
pub struct FlantermContext {
    ctx: NonNull<flanterm_context>,
}

// SAFETY: the context is only ever reached through the mutex in `FT_CTX`, which serialises every
// call into Flanterm, and Flanterm keeps no per-thread or per-LP state of its own.
unsafe impl Send for FlantermContext {}
unsafe impl Sync for FlantermContext {}

impl FlantermContext {
    pub fn new(ctx: NonNull<flanterm_context>) -> Self {
        FlantermContext {
            ctx,
        }
    }
}

impl Write for FlantermContext {
    /// Writes a string to the terminal verbatim, with no newline translation.
    ///
    /// A line feed is a line feed and nothing more: it advances one row and leaves the column
    /// where it was. Anything that wants the next line to begin at column zero must say so with an
    /// explicit carriage return, so line breaks headed for the terminal are written `\r\n`. See
    /// the note on [`FlantermContext`].
    fn write_str(&mut self, str: &str) -> core::fmt::Result {
        // SAFETY: `ctx` was produced by `flanterm_fb_init` and has not been deinitialised, and
        // `str` is a live allocation of exactly `str.len()` bytes.
        unsafe {
            flanterm_write(self.ctx.as_ptr(), str.as_ptr() as *const c_char, str.len());
        }

        Ok(())
    }
}

static CONTEXT_READY: AtomicBool = AtomicBool::new(false);

pub fn context() -> &'static Mutex<FlantermContext> {
    let context = &*FT_CTX;
    CONTEXT_READY.store(true, Ordering::Release);
    context
}

/// Panic reporting must neither initialize the framebuffer nor wait for its lock.
pub fn initialized_context() -> Option<&'static Mutex<FlantermContext>> {
    CONTEXT_READY.load(Ordering::Acquire).then(|| &*FT_CTX)
}

pub static FT_CTX: LazyLock<Mutex<FlantermContext>> = LazyLock::new(|| {
    let fb_res = FRAMEBUFFER_REQUEST.response().expect("Failed to get Limine framebuffer response");
    let fb = fb_res.framebuffers().first().expect("No framebuffer found in Limine response");
    // SAFETY: the framebuffer geometry comes straight from Limine, the allocator callbacks are
    // valid for the lifetime of the kernel, and the null arguments select Flanterm's defaults for
    // the canvas, the ANSI palette and the default colours.
    let ctx = unsafe {
        flanterm_fb_init(
            Some(malloc),
            Some(free),
            fb.address() as *mut u32,
            fb.width as usize,
            fb.height as usize,
            fb.pitch as usize,
            fb.red_mask_size,
            fb.red_mask_shift,
            fb.green_mask_size,
            fb.green_mask_shift,
            fb.blue_mask_size,
            fb.blue_mask_shift,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            &raw const super::chars::FONT as *mut c_void,
            FONT_WIDTH,
            FONT_HEIGHT,
            0,
            0,
            0,
            0,
            FLANTERM_FB_ROTATE_0,
        )
    };

    let ctx = NonNull::new(ctx).expect("Flanterm failed to initialize a framebuffer context");
    Mutex::new(FlantermContext::new(ctx))
});

pub extern "C" fn malloc(size: usize) -> *mut c_void {
    let layout = core::alloc::Layout::from_size_align(size, MALLOC_ALIGN).unwrap();
    unsafe { alloc::alloc::alloc(layout) as *mut c_void }
}

pub extern "C" fn free(ptr: *mut c_void, size: usize) {
    let layout = core::alloc::Layout::from_size_align(size, MALLOC_ALIGN).unwrap();
    unsafe { alloc::alloc::dealloc(ptr as *mut u8, layout) };
}
