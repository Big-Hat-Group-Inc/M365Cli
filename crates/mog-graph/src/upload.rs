/// Upload session management for resumable file uploads
use mog_core::error::MogError;

/// Default chunk size: 5MB
pub const DEFAULT_CHUNK_SIZE: u64 = 5 * 1024 * 1024; // 5,242,880

/// Minimum chunk size per Graph requirement: 320KB
pub const MIN_CHUNK_SIZE: u64 = 320 * 1024;

/// Maximum chunk size: 60MB
pub const MAX_CHUNK_SIZE: u64 = 60 * 1024 * 1024;

/// Validate chunk size
pub fn validate_chunk_size(size: u64) -> Result<u64, MogError> {
    if size < MIN_CHUNK_SIZE {
        return Err(MogError::Validation(format!(
            "Chunk size {} is below minimum {} (320KB)",
            size, MIN_CHUNK_SIZE
        )));
    }
    if size > MAX_CHUNK_SIZE {
        return Err(MogError::Validation(format!(
            "Chunk size {} exceeds maximum {} (60MB)",
            size, MAX_CHUNK_SIZE
        )));
    }
    // Must be a multiple of 320KB per Graph requirement
    let aligned = (size / MIN_CHUNK_SIZE) * MIN_CHUNK_SIZE;
    if aligned != size {
        tracing::warn!(
            "Chunk size {} is not a multiple of 320KB; aligning to {}",
            size,
            aligned
        );
    }
    Ok(aligned)
}

/// Upload session state for resume support
#[derive(Debug, Clone)]
pub struct UploadSession {
    pub upload_url: String,
    pub expiration: Option<String>,
    pub total_size: u64,
    pub bytes_uploaded: u64,
    pub chunk_size: u64,
}

impl UploadSession {
    pub fn new(upload_url: String, total_size: u64, chunk_size: u64) -> Self {
        Self {
            upload_url,
            expiration: None,
            total_size,
            bytes_uploaded: 0,
            chunk_size,
        }
    }

    /// Calculate the Content-Range header for the next chunk
    pub fn content_range(&self) -> String {
        let start = self.bytes_uploaded;
        let end = std::cmp::min(start + self.chunk_size, self.total_size) - 1;
        format!("bytes {}-{}/{}", start, end, self.total_size)
    }

    /// How many bytes remain
    pub fn remaining(&self) -> u64 {
        self.total_size.saturating_sub(self.bytes_uploaded)
    }

    /// Size of the next chunk
    pub fn next_chunk_size(&self) -> u64 {
        std::cmp::min(self.chunk_size, self.remaining())
    }

    /// Whether upload is complete
    pub fn is_complete(&self) -> bool {
        self.bytes_uploaded >= self.total_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_chunk_size() {
        assert!(validate_chunk_size(MIN_CHUNK_SIZE).is_ok());
        assert!(validate_chunk_size(DEFAULT_CHUNK_SIZE).is_ok());
        assert!(validate_chunk_size(100).is_err());
        assert!(validate_chunk_size(MAX_CHUNK_SIZE + 1).is_err());
    }

    #[test]
    fn test_upload_session_content_range() {
        let session = UploadSession::new("http://example.com".into(), 10_000_000, 5_000_000);
        assert_eq!(session.content_range(), "bytes 0-4999999/10000000");
    }

    #[test]
    fn test_upload_session_progress() {
        let mut session = UploadSession::new("http://example.com".into(), 10_000_000, 5_000_000);
        assert_eq!(session.remaining(), 10_000_000);
        session.bytes_uploaded = 5_000_000;
        assert_eq!(session.remaining(), 5_000_000);
        assert_eq!(session.next_chunk_size(), 5_000_000);
        session.bytes_uploaded = 10_000_000;
        assert!(session.is_complete());
    }

    #[test]
    fn test_validate_chunk_size_boundaries() {
        // Exactly min
        assert_eq!(validate_chunk_size(MIN_CHUNK_SIZE).unwrap(), MIN_CHUNK_SIZE);
        // Exactly max
        assert_eq!(validate_chunk_size(MAX_CHUNK_SIZE).unwrap(), MAX_CHUNK_SIZE);
        // Below min
        assert!(validate_chunk_size(100).is_err());
        // Above max
        assert!(validate_chunk_size(MAX_CHUNK_SIZE + 1).is_err());
    }

    #[test]
    fn test_validate_chunk_size_alignment() {
        // 1MB is a multiple of 320KB? 1048576 / 327680 = 3.2 → aligned to 3*320KB = 983040
        let result = validate_chunk_size(1_048_576).unwrap();
        assert_eq!(result % MIN_CHUNK_SIZE, 0);
    }

    #[test]
    fn test_default_chunk_size_valid() {
        const { assert!(DEFAULT_CHUNK_SIZE >= MIN_CHUNK_SIZE) };
        const { assert!(DEFAULT_CHUNK_SIZE <= MAX_CHUNK_SIZE) };
        assert_eq!(DEFAULT_CHUNK_SIZE, 5 * 1024 * 1024);
    }

    #[test]
    fn test_content_range_last_chunk() {
        let mut session = UploadSession::new("http://example.com".into(), 7_000_000, 5_000_000);
        session.bytes_uploaded = 5_000_000;
        // Last chunk should be 2MB
        assert_eq!(session.next_chunk_size(), 2_000_000);
        assert_eq!(session.content_range(), "bytes 5000000-6999999/7000000");
    }
}
