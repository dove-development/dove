use solana_program::pubkey::Pubkey;

use crate::util::require;

/// A trait representing a "plain old data" (POD) type.
///
/// ## Safety
/// - The type must be able to represent any possible bit pattern of its underlying byte array.
/// - This ensures that any sequence of bytes can be safely interpreted as this type.
///
/// ## Note
/// - This trait can cast from any alignment, as both the Solana and WASM runtimes support unaligned memory accesses.
pub unsafe trait Pod: Sized + Copy {
    const NAME: &'static str = "object";
    const SIZE: usize = std::mem::size_of::<Self>();
    fn zero() -> Self {
        unsafe { std::mem::zeroed() }
    }
    fn cast_from(bytes: &[u8]) -> &Self {
        require(bytes.len() >= Self::SIZE, "can't cast to const object (length too short)");
        unsafe { &*(bytes.as_ptr() as *const Self) }
    }
    fn cast_from_mut(bytes: &mut [u8]) -> &mut Self {
        require(bytes.len() >= Self::SIZE, "can't cast to mut object (length too short)");
        unsafe { &mut *(bytes.as_mut_ptr() as *mut Self) }
    }
    fn try_cast_from(bytes: &[u8]) -> Result<&Self, &'static str> {
        if bytes.len() >= Self::SIZE {
            Ok(unsafe { &*(bytes.as_ptr() as *const Self) })
        } else {
            Err("can't try-cast to const object (length too short)")
        }
    }
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, Self::SIZE) }
    }
}

unsafe impl Pod for Pubkey {
    const NAME: &'static str = "Pubkey";
}
