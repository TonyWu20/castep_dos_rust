use std::array::TryFromSliceError;

use thiserror::Error;
use winnow::{
    Parser,
    binary::be_u32,
    error::{ContextError, InputError, StrContext},
    token::take,
};

#[derive(Debug, Error)]
/// Errors in this module
pub enum HelperError {
    #[error("Failed to convert to `<[u8;N]>`")]
    /// Error when parsed `&[u8]` convert to designated length array [u8; N] (N = 4 or 8)
    BytesIntoArray(#[from] TryFromSliceError),
    #[error("Parser error: {0}")]
    /// Error from calling `winnow` parsers
    ParserContextError(winnow::error::ErrMode<ContextError>),
    #[error("Parser Input error ")]
    /// Error from calling be_u32
    ParserInputError(InputError<Vec<u8>>),
}

impl From<winnow::error::ErrMode<ContextError>> for HelperError {
    fn from(value: winnow::error::ErrMode<ContextError>) -> Self {
        Self::ParserContextError(value)
    }
}

impl From<InputError<&[u8]>> for HelperError {
    fn from(value: InputError<&[u8]>) -> Self {
        Self::ParserInputError(InputError {
            input: value.input.to_vec(),
        })
    }
}

/// Helper: peek the size of next record without consuming
/// the output.
/// Used to distinguish the headers of `.pdos_weights`and `.pdos_bin`
/// - `.pdos_weights`: starts with a `u32` integer for num of total k-points
/// - `.pdos_bin` : starts with an `f64` as version number, and then a line
///   of string specifying `CASTEP` version and generated date.
pub(crate) fn peek_record<'a>(input: &mut &'a [u8]) -> Result<(&'a [u8], usize), HelperError> {
    take::<usize, &[u8], InputError<&[u8]>>(4_usize)
        .verify_map(|bytes| bytes.try_into().ok())
        .map(|bytes| u32::from_be_bytes(bytes) as usize)
        .parse_peek(input)
        .map_err(HelperError::from)
}

/// Helper: parse big-endian record with size validation.
/// # Errors
///
/// This function will return an error if .
pub(crate) fn parse_record<'a>(
    input: &mut &'a [u8],
    expected_size: usize,
) -> Result<&'a [u8], HelperError> {
    // Use `verify` to ensure the parsed record size is as expected
    let record_size = be_u32::<&'a [u8], InputError<&'a [u8]>>
        .verify(|size| *size == expected_size as u32)
        .context(StrContext::Label("starting record size marker"))
        .context(StrContext::Expected(
            winnow::error::StrContextValue::Description("record size mismatch with expected size"),
        ))
        .parse_next(input)?;

    let data = take::<_, &[u8], InputError<&[u8]>>(record_size).parse_next(input)?;
    let _end_marker = be_u32::<&'a [u8], InputError<&'a [u8]>>
        .verify(|size| *size == record_size)
        .context(StrContext::Label("ending record size marker"))
        .context(StrContext::Expected(
            winnow::error::StrContextValue::Description(
                "ending record size mismatch with starting record marker size",
            ),
        ))
        .parse_next(input)?;
    Ok(data)
}

/// Helper functions to parse `u32` or `f64`
pub(crate) fn parse_scalar<T, const N: usize>(input: &mut &[u8]) -> Result<T, HelperError>
where
    T: FromBeBytes<N>,
{
    let data = parse_record(input, N)?;
    Ok(T::from_be_bytes(
        data.try_into().map_err(HelperError::BytesIntoArray)?,
    ))
}

pub(crate) fn parse_vec<T, const N: usize>(
    input: &mut &[u8],
    len: usize,
) -> Result<Vec<T>, HelperError>
where
    T: FromBeBytes<N>,
{
    let data = parse_record(input, N * len)?;
    data.chunks_exact(N)
        .map(|chunk| {
            Ok(T::from_be_bytes(
                chunk.try_into().map_err(HelperError::BytesIntoArray)?,
            ))
        })
        .collect::<Result<Vec<T>, HelperError>>()
}

pub(crate) trait FromBeBytes<const N: usize>: Sized {
    fn from_be_bytes(bytes: [u8; N]) -> Self;
}

impl FromBeBytes<4> for u32 {
    fn from_be_bytes(bytes: [u8; 4]) -> Self {
        Self::from_be_bytes(bytes)
    }
}

impl FromBeBytes<8> for f64 {
    fn from_be_bytes(bytes: [u8; 8]) -> Self {
        Self::from_be_bytes(bytes)
    }
}
