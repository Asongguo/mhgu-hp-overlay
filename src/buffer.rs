use core::fmt::Write;

/// 简易格式化缓冲（no_std 环境下的轻量字符串缓冲）
pub struct StringBuffer<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> StringBuffer<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn set_str(&mut self, s: &str) {
        self.clear();
        let _ = core::fmt::Write::write_str(self, s);
    }
}

impl<const N: usize> Clone for StringBuffer<N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<const N: usize> Copy for StringBuffer<N> {}

impl<const N: usize> Write for StringBuffer<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = N - self.len;
        let to_copy = bytes.len().min(remaining);
        self.buf[self.len..self.len + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.len += to_copy;
        Ok(())
    }
}
