use ark_ec::PairingEngine;
use ark_poly::EvaluationDomain;

use crate::{
    kzg10::{
        commit_key_lag::lagrange::{vec_add_scalar, LagrangeBasis},
        errors::KZG10Error,
        proof::AggregateProofMultiPoint,
        CommitKeyLagrange, Commitment, LagrangeCommitter, MultiPointProver,
    },
    transcript::TranscriptProtocol,
    util::powers_of,
};

impl<E: PairingEngine, T: TranscriptProtocol<E>> MultiPointProver<E, T> for CommitKeyLagrange<E> {
    fn open_multipoint_lagrange(
        &self,
        lagrange_polynomials: &[ark_poly::Evaluations<E::Fr>],
        poly_commitments: Option<&[Commitment<E>]>,
        evaluations: &[E::Fr],
        points: &[E::Fr], // These will be roots of unity
        transcript: &mut T,
    ) -> Result<AggregateProofMultiPoint<E>, KZG10Error> {
        let num_polynomials = lagrange_polynomials.len();

        let domain = lagrange_polynomials
            .first()
            .expect("expected at least one polynomial")
            .domain();
        let domain_size = domain.size();

        // Commit to polynomials, if not done so already
        match poly_commitments {
            None => {
                for poly in lagrange_polynomials.iter() {
                    let poly_commit = LagrangeCommitter::commit_lagrange(self, &poly.evals)?;
                    transcript.append_point(b"f_x", &poly_commit.0);
                }
            }
            Some(commitments) => {
                for poly_commit in commitments.iter() {
                    transcript.append_point(b"f_x", &poly_commit.0);
                }
            }
        };

        for point in points {
            transcript.append_scalar(b"value", point)
        }

        for point in evaluations {
            transcript.append_scalar(b"eval", point)
        }

        // compute the witness for each polynomial at their respective points
        use rayon::prelude::*;
        let domain_elements: Vec<_> = domain.elements().collect();
        let inv = Self::compute_inv(&domain_elements);

        // Compute a new polynomial which sums together all of the witnesses for each polynomial
        // aggregate the witness polynomials to form the new polynomial that we want to run KZG10 on
        let r = transcript.challenge_scalar(b"r");
        let r_i = powers_of::<E::Fr>(&r, num_polynomials - 1);

        let each_witness = lagrange_polynomials
            .into_par_iter()
            .zip(points)
            .zip(evaluations)
            .map(|((poly, point), evaluation)| {
                let lb = LagrangeBasis::<E>::from(poly).add_scalar(&evaluation);

                let witness_poly = LagrangeBasis::<E>::divide_by_linear_vanishing_from_point(
                    point,
                    &lb.0,
                    &inv,
                    &domain_elements,
                );
                witness_poly
            });

        let g_x: LagrangeBasis<E> = each_witness
            .zip(r_i.par_iter())
            .map(|(poly, challenge)| poly * challenge)
            .fold(|| LagrangeBasis::zero(domain_size), |res, val| res + val)
            .reduce(|| LagrangeBasis::zero(domain_size), |res, val| res + val);

        // Commit to to this poly_sum witness
        let d_comm = LagrangeCommitter::commit_lagrange(self, g_x.values())?;

        transcript.append_scalar(b"r", &r);
        transcript.append_point(b"D", &d_comm.0);

        // Compute new point to evaluate g_x at
        let t = transcript.challenge_scalar(b"t");
        // compute the helper polynomial which will help the verifier compute g(t)
        //
        let mut denominator: Vec<_> = points.par_iter().map(|z_i| t - z_i).collect();
        ark_ff::batch_inversion(&mut denominator);
        let helper_coefficients = r_i
            .into_par_iter()
            .zip(denominator)
            .map(|(r_i, den)| r_i * den);

        let h_x: LagrangeBasis<E> = helper_coefficients
            .zip(lagrange_polynomials.par_iter())
            .map(|(helper_scalars, poly)| LagrangeBasis::from(poly) * &helper_scalars)
            .fold(|| LagrangeBasis::zero(domain_size), |res, val| res + val)
            .reduce(|| LagrangeBasis::zero(domain_size), |res, val| res + val);

        // XXX: The prover only computes the commitment to add it to the transcript
        // Can we remove this, and say that since h_t is added to the transcript
        // then this is fine?
        let E = LagrangeCommitter::commit_lagrange(self, &h_x.values())?;

        // Evaluate both polynomials at the point `t`
        let h_t = h_x.evaluate_point_outside_domain(&t);
        let g_t = g_x.evaluate_point_outside_domain(&t);

        // We can now aggregate both proofs into an aggregate proof

        transcript.append_point(b"E", &E.0);
        transcript.append_point(b"d_comm", &d_comm.0);
        transcript.append_scalar(b"h_t", &h_t);
        transcript.append_scalar(b"g_t", &g_t);

        let sum_quotient = d_comm;
        let helper_evaluation = h_t;
        let aggregated_witness_poly = self.compute_aggregate_witness_lagrange(
            vec![h_x.0, g_x.0],
            &t,
            transcript,
            &domain_elements,
        );
        let aggregated_witness =
            LagrangeCommitter::commit_lagrange(self, &aggregated_witness_poly.values())?;

        Ok(AggregateProofMultiPoint {
            sum_quotient,
            helper_evaluation,
            aggregated_witness,
        })
    }
}
