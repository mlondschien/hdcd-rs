use crate::gain::GainResult;
use crate::optimizer::OptimizerResult;
use crate::{Control, Gain, Optimizer};

pub struct GridSearch<'a, T: Gain> {
    pub gain: T,
    pub control: &'a Control,
}

impl<'a, T> Optimizer for GridSearch<'a, T>
where
    T: Gain,
{
    fn n(&self) -> usize {
        self.gain.n()
    }

    fn control(&self) -> &Control {
        self.control
    }

    fn find_best_split(&self, start: usize, stop: usize) -> Result<OptimizerResult, &str> {
        let split_candidates = self.split_candidates(start, stop);

        if split_candidates.is_empty() {
            return Err("Segment too small.");
        }

        let full_gain = self.gain.gain_full(start, stop, &split_candidates);

        let mut best_split = 0;
        let mut max_gain = -f64::INFINITY;

        for index in split_candidates {
            if full_gain.gain[index - start] > max_gain {
                best_split = index;
                max_gain = full_gain.gain[index - start];
            }
        }

        Ok(OptimizerResult {
            start,
            stop,
            best_split,
            max_gain,
            gain_results: vec![GainResult::FullGainResult(full_gain)],
        })
    }

    fn is_significant(&self, optimizer_result: &OptimizerResult) -> bool {
        let gain_result = optimizer_result.gain_results.last().unwrap();
        self.gain
            .is_significant(optimizer_result.max_gain, gain_result, self.control())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::testing;
    use rstest::*;

    #[rstest]
    #[case(0, 7, 2)]
    #[case(1, 7, 4)]
    #[case(2, 7, 4)]
    #[case(3, 7, 4)]
    #[case(1, 5, 2)]
    #[case(1, 6, 4)]
    #[case(1, 7, 4)]
    #[case(2, 6, 4)]
    #[case(2, 7, 4)]
    #[case(3, 6, 4)]
    fn test_change_in_mean_find_best_split(
        #[case] start: usize,
        #[case] stop: usize,
        #[case] expected: usize,
    ) {
        let X = ndarray::array![
            [0., 1.],
            [0., 1.],
            [1., -1.],
            [1., -1.],
            [-1., -1.],
            [-1., -1.],
            [-1., -1.]
        ];
        let X_view = X.view();
        assert_eq!(X_view.shape(), &[7, 2]);

        let gain = testing::ChangeInMean::new(&X_view);
        let control = Control::default().with_minimal_relative_segment_length(0.1);
        let grid_search = GridSearch {
            gain,
            control: &control,
        };
        assert_eq!(
            grid_search.find_best_split(start, stop).unwrap().best_split,
            expected
        );
    }
}
