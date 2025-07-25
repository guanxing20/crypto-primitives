use super::CommitmentScheme;
pub use crate::crh::pedersen::Window;
use crate::{
    crh::{pedersen, CRHScheme},
    Error,
};
use ark_ec::CurveGroup;
use ark_ff::{BitIteratorLE, Field, PrimeField, ToConstraintField};
use ark_serialize::CanonicalSerialize;
#[cfg(not(feature = "std"))]
use ark_std::vec::Vec;
use ark_std::{marker::PhantomData, rand::Rng, UniformRand};

#[cfg(feature = "constraints")]
pub mod constraints;

#[derive(Clone)]
pub struct Parameters<C: CurveGroup> {
    pub randomness_generator: Vec<C>,
    pub generators: Vec<Vec<C>>,
}

pub struct Commitment<C: CurveGroup, W: Window> {
    group: PhantomData<C>,
    window: PhantomData<W>,
}

#[derive(Derivative, CanonicalSerialize)]
#[derivative(Clone, PartialEq, Debug, Eq, Default)]
pub struct Randomness<C: CurveGroup>(pub C::ScalarField);

impl<C: CurveGroup> UniformRand for Randomness<C> {
    #[inline]
    fn rand<R: Rng + ?Sized>(rng: &mut R) -> Self {
        Randomness(UniformRand::rand(rng))
    }
}

impl<C: CurveGroup, W: Window> CommitmentScheme for Commitment<C, W> {
    type Parameters = Parameters<C>;
    type Randomness = Randomness<C>;
    type Output = C::Affine;

    fn setup<R: Rng>(rng: &mut R) -> Result<Self::Parameters, Error> {
        let time = start_timer!(|| format!(
            "PedersenCOMM::Setup: {} {}-bit windows; {{0,1}}^{{{}}} -> C",
            W::NUM_WINDOWS,
            W::WINDOW_SIZE,
            W::NUM_WINDOWS * W::WINDOW_SIZE
        ));
        let num_powers = <C::ScalarField as PrimeField>::MODULUS_BIT_SIZE as usize;
        let randomness_generator = pedersen::CRH::<C, W>::generator_powers(num_powers, rng);
        let generators = pedersen::CRH::<C, W>::create_generators(rng);
        end_timer!(time);

        Ok(Self::Parameters {
            randomness_generator,
            generators,
        })
    }

    fn commit(
        parameters: &Self::Parameters,
        input: &[u8],
        randomness: &Self::Randomness,
    ) -> Result<Self::Output, Error> {
        let commit_time = start_timer!(|| "PedersenCOMM::Commit");
        // If the input is too long, return an error.
        if input.len() > W::WINDOW_SIZE * W::NUM_WINDOWS {
            panic!("incorrect input length: {:?}", input.len());
        }
        // Pad the input to the necessary length.
        let mut padded_input = Vec::with_capacity(input.len());
        let mut input = input;
        if (input.len() * 8) < W::WINDOW_SIZE * W::NUM_WINDOWS {
            padded_input.extend_from_slice(input);
            let padded_length = (W::WINDOW_SIZE * W::NUM_WINDOWS) / 8;
            padded_input.resize(padded_length, 0u8);
            input = padded_input.as_slice();
        }
        assert_eq!(parameters.generators.len(), W::NUM_WINDOWS);
        let input = input.to_vec();
        // Invoke Pedersen CRH here, to prevent code duplication.

        let crh_parameters = pedersen::Parameters {
            generators: parameters.generators.clone(),
        };
        let mut result: C =
            pedersen::CRH::<C, W>::evaluate(&crh_parameters, input.as_slice())?.into();
        let randomize_time = start_timer!(|| "Randomize");

        // Compute h^r.
        for (bit, power) in BitIteratorLE::new(randomness.0.into_bigint())
            .into_iter()
            .zip(&parameters.randomness_generator)
        {
            if bit {
                result += power
            }
        }
        end_timer!(randomize_time);
        end_timer!(commit_time);

        Ok(result.into())
    }
}

impl<ConstraintF: Field, C: CurveGroup + ToConstraintField<ConstraintF>>
    ToConstraintField<ConstraintF> for Parameters<C>
{
    #[inline]
    fn to_field_elements(&self) -> Option<Vec<ConstraintF>> {
        Some(Vec::new())
    }
}
