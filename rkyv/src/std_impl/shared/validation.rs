use core::{any::TypeId, fmt};
use std::error;
use bytecheck::CheckBytes;
use super::ArchivedRc;
use crate::validation::{ArchiveBoundsContext, ArchiveBoundsError, CheckBytesRef, SharedArchiveContext, SharedArchiveError};

#[derive(Debug)]
pub enum SharedPointerError<T, R> {
    BoundsError(ArchiveBoundsError),
    SharedError(SharedArchiveError),
    CheckBytes(T),
    RefCheckBytes(R),
}

impl<T, R> From<ArchiveBoundsError> for SharedPointerError<T, R> {
    fn from(e: ArchiveBoundsError) -> Self {
        Self::BoundsError(e)
    }
}

impl<T, R> From<SharedArchiveError> for SharedPointerError<T, R> {
    fn from(e: SharedArchiveError) -> Self {
        Self::SharedError(e)
    }
}

impl<T: fmt::Display, R: fmt::Display> fmt::Display for SharedPointerError<T, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SharedPointerError::BoundsError(e) => write!(f, "{}", e),
            SharedPointerError::SharedError(e) => write!(f, "{}", e),
            SharedPointerError::CheckBytes(e) => write!(f, "{}", e),
            SharedPointerError::RefCheckBytes(e) => write!(f, "{}", e),
        }
    }
}

impl<T: fmt::Debug + fmt::Display, R: fmt::Debug + fmt::Display> error::Error for SharedPointerError<T, R> {}

impl<T: CheckBytesRef<C> + 'static, C: ArchiveBoundsContext + SharedArchiveContext + ?Sized> CheckBytes<C> for ArchivedRc<T> {
    type Error = SharedPointerError<T::Error, T::RefError>;

    unsafe fn check_bytes<'a>(bytes: *const u8, context: &mut C) -> Result<&'a Self, Self::Error> {
        let reference = T::check_bytes(bytes, context).map_err(SharedPointerError::CheckBytes)?;
        let (start, len) = reference.check_ptr(context)?;
        if let Some(ref_bytes) = context.claim_shared_bytes(start, len, TypeId::of::<T>())? {
            reference.check_ref_bytes(ref_bytes, context).map_err(SharedPointerError::RefCheckBytes)?;
        }
        Ok(&*bytes.cast())
    }
}