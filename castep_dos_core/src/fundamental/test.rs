use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::{
    AngularMomentum, EigenvalueVec, KpointVec, OrbitalState, OrbitalWeight, OrbitalWeightVec,
    PDOSWeights, SpinData, SpinIndex, SpinPolarized,
};
/// Gather weights at each eigenvalue's orbital weight array
fn sum_weights_per_eigenvalue(indices: &[usize], weights: &OrbitalWeightVec) -> f64 {
    indices.par_iter().map(|&idx| weights[idx].value()).sum()
}

#[test]
fn defined_types() {
    // Construction - matches your exact specification
    let pdos_weights = PDOSWeights {
        spin_polarized: SpinPolarized::True,
        orbital_states: vec![OrbitalState::new(1, 1, AngularMomentum::S)],
        orbital_weights: SpinData::SpinPolarized([
            KpointVec::new(vec![EigenvalueVec::new(vec![
                OrbitalWeightVec::new(vec![OrbitalWeight::new(0.1), OrbitalWeight::new(0.2)]),
                OrbitalWeightVec::new(vec![OrbitalWeight::new(0.3), OrbitalWeight::new(0.4)]),
            ])]),
            KpointVec::new(vec![EigenvalueVec::new(vec![
                OrbitalWeightVec::new(vec![OrbitalWeight::new(0.5), OrbitalWeight::new(0.6)]),
                OrbitalWeightVec::new(vec![OrbitalWeight::new(0.7), OrbitalWeight::new(0.8)]),
            ])]),
        ]),
    };

    // Access patterns - direct and intuitive
    let first_kpoint = &pdos_weights.orbital_weights.get(SpinIndex::One).unwrap()[0];
    let first_eigenvalue = &first_kpoint[0];
    let _first_orbital = first_eigenvalue[0].value();

    // Iteration - using Deref to Vec
    for kpoint_data in pdos_weights
        .orbital_weights
        .get(SpinIndex::One)
        .unwrap()
        .iter()
    {
        for eigenvalue_data in kpoint_data.iter() {
            for orbital_weight in eigenvalue_data.iter() {
                println!("Weight: {}", orbital_weight.value());
            }
        }
    }
    // Iteration - using Deref to Vec
    pdos_weights
        .orbital_weights
        .get(SpinIndex::One)
        .map(|spin_channel| {
            spin_channel.iter().for_each(|kpt_data| {
                kpt_data.iter().for_each(|eigen_data| {
                    eigen_data
                        .iter()
                        .for_each(|weight| println!("Weight: {}", weight.value()));
                });
            });
        })
        .unwrap();

    let indices = vec![0_usize, 1_usize];
    // Functional transformation
    let _squared_weights = pdos_weights
        .orbital_weights
        .map_on_data_of_eigenvalue(|w| sum_weights_per_eigenvalue(&indices, w));
}
