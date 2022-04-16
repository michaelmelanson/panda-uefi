use crate::{acpi::AcpiError, logger::LoggerError, memory::MemoryError};
use panda_loader_lib::LoaderCarePackageError;

#[derive(Debug)]
pub enum KernelError {
    AcpiError(AcpiError),
    LoggerError(LoggerError),
    LoaderCarePackageError(LoaderCarePackageError),
    MemoryError(MemoryError),
}

impl From<AcpiError> for KernelError {
    fn from(error: AcpiError) -> Self {
        KernelError::AcpiError(error)
    }
}

impl From<LoggerError> for KernelError {
    fn from(error: LoggerError) -> Self {
        KernelError::LoggerError(error)
    }
}

impl From<LoaderCarePackageError> for KernelError {
    fn from(error: LoaderCarePackageError) -> Self {
        KernelError::LoaderCarePackageError(error)
    }
}

impl From<MemoryError> for KernelError {
    fn from(error: MemoryError) -> Self {
        KernelError::MemoryError(error)
    }
}
