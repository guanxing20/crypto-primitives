use crate::Error;
use ark_std::rand::Rng;

#[cfg(feature = "constraints")]
pub mod constraints;
pub mod elgamal;
#[cfg(feature = "constraints")]
pub use constraints::*;

pub trait AsymmetricEncryptionScheme {
    type Parameters;
    type PublicKey;
    type SecretKey;
    type Randomness;
    type Plaintext;
    type Ciphertext;

    fn setup<R: Rng>(rng: &mut R) -> Result<Self::Parameters, Error>;

    fn keygen<R: Rng>(
        pp: &Self::Parameters,
        rng: &mut R,
    ) -> Result<(Self::PublicKey, Self::SecretKey), Error>;

    fn encrypt(
        pp: &Self::Parameters,
        pk: &Self::PublicKey,
        message: &Self::Plaintext,
        r: &Self::Randomness,
    ) -> Result<Self::Ciphertext, Error>;

    fn decrypt(
        pp: &Self::Parameters,
        sk: &Self::SecretKey,
        ciphertext: &Self::Ciphertext,
    ) -> Result<Self::Plaintext, Error>;
}
