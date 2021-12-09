use crate::control::Control;
use crate::gain::{ApproxGain, ApproxGainResult, Gain, GainResult};
use crate::optimizer::OptimizerResult;
use crate::Classifier;
use crate::ModelSelectionResult;
use ndarray::{s, Array1, Array2, Axis};
use rand::{rngs::StdRng, SeedableRng};

pub struct ClassifierGain<T: Classifier> {
    pub classifier: T,
}

// impl<T> ClassifierGain<T>
// where
//     T: Classifier,
// {
//     fn single_(&self, gain_result: &GainResult, rng: &mut StdRng) -> Self {
//         let likelihoods: &Array2<f64>;
//         let start: usize;
//         let stop: usize;

//         if let GainResult::ApproxGainResult(result) = gain_result {
//             likelihoods = &result.likelihoods;
//             start = result.start;
//             stop = result.stop;
//         } else {
//             panic!();
//         }

//         Self {
//             classifier,
//         }
//     }
// }

impl<T> Gain for ClassifierGain<T>
where
    T: Classifier,
{
    /// Total number of observations.
    fn n(&self) -> usize {
        self.classifier.n()
    }

    /// Return classifier-likelihood based gain when splitting segment `[start, stop)`
    /// at `split`.
    fn gain(&self, start: usize, stop: usize, split: usize) -> f64 {
        let predictions = self.classifier.predict(start, stop, split);
        self.classifier
            .single_likelihood(&predictions, start, stop, split)
    }

    fn model_selection(&self, optimizer_result: &OptimizerResult) -> ModelSelectionResult {
        let mut rng = StdRng::seed_from_u64(self.control().seed);
        let n_permutations = 99;

        let mut max_gain = -f64::INFINITY;
        let mut deltas: Vec<Array1<f64>> = Vec::with_capacity(3);
        let mut likelihood_0: Vec<f64> = Vec::with_capacity(3);

        for jdx in 0..3 {
            let result = match &optimizer_result.gain_results[jdx] {
                GainResult::ApproxGainResult(result) => result,
                _ => panic!("Not an ApproxGainResult"),
            };

            deltas
                .push(&result.likelihoods.slice(s![0, ..]) - &result.likelihoods.slice(s![1, ..]));
            likelihood_0.push(result.likelihoods.slice(s![1, ..]).sum());
            if result.max_gain.unwrap() > max_gain {
                max_gain = result.max_gain.unwrap();
            }
        }

        let mut p_value: u32 = 1;
        let segment_length = optimizer_result.stop - optimizer_result.start;

        for _ in 0..n_permutations {
            let mut values = likelihood_0.clone();

            // Test if for any jdx=1,2,3 the gain (likelihood_0[jdx] + cumsum(deltas[jdx]))
            // is greater than max_gain. This is the statistic we are comparing against.
            'outer: for idx in rand::seq::index::sample(&mut rng, segment_length, segment_length) {
                for jdx in 0..3 {
                    values[jdx] += deltas[jdx][idx];
                    if values[jdx] >= optimizer_result.max_gain {
                        p_value += 1;
                        // break both loops. We only need to check if the maximum of the
                        // maximal gain after permutation is ever greater than the
                        // original max_gain (without permutation).
                        break 'outer;
                    }
                }
            }
        }

        // Up to here p_value is # of permutations for which the max_gain is higher than
        // the non-permuted max_gain. From this create a true p_value.
        let p_value = p_value as f64 / (n_permutations + 1) as f64;
        let is_significant = p_value < self.control().model_selection_alpha;

        ModelSelectionResult {
            is_significant,
            p_value: Some(p_value),
        }
    }

    fn control(&self) -> &Control {
        self.classifier.control()
    }
}

impl<T> ApproxGain for ClassifierGain<T>
where
    T: Classifier,
{
    /// Return an approximation of the classifier-likelihood based gain when splitting
    /// segment `[start, stop)` for each split in `split_candidates`.
    ///
    /// A single fit is generated with a split at `guess`.
    fn gain_approx(
        &self,
        start: usize,
        stop: usize,
        guess: usize,
        _: &[usize],
    ) -> ApproxGainResult {
        let predictions = self.classifier.predict(start, stop, guess);
        let likelihoods = self
            .classifier
            .full_likelihood(&predictions, start, stop, guess);

        let gain = gain_from_likelihoods(&likelihoods);

        ApproxGainResult {
            start,
            stop,
            guess,
            gain,
            best_split: None,
            max_gain: None,
            likelihoods,
            predictions,
        }
    }
}

pub fn gain_from_likelihoods(likelihoods: &Array2<f64>) -> Array1<f64> {
    let n = likelihoods.shape()[1];
    let mut gain = Array1::<f64>::zeros(n);
    // Move everything one to the right.
    gain.slice_mut(s![1..])
        .assign(&(&likelihoods.slice(s![0, ..(n - 1)]) - &likelihoods.slice(s![1, ..(n - 1)])));
    gain.accumulate_axis_inplace(Axis(0), |&prev, curr| *curr += prev);

    gain + likelihoods.slice(s![1, ..]).sum()
}
