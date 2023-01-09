use std::f64::consts::PI;

use anyhow::{bail, Error, Result};
/// To perform an ANOVA test in Rust, you will need to:
///
/// Make sure that your data meet the assumptions for an ANOVA test. These include:
///     Normality: The data should be approximately normally distributed within each group.
///     Independence: The observations should be independent of one another.
///     Equal variances: The variances of the groups should be approximately equal.
///
/// Identify the variables you want to compare. In a polar dataframe, these might be the angles or the magnitudes at different points in time.
///
/// Split the data into the appropriate groups. This will depend on the research question you are trying to answer.
/// For example, you might want to compare the angles at different times for different subjects, or you might want to compare the magnitudes at different times for different conditions.
///
/// Calculate the means and variances for each group.
///
/// Use the formulas for an ANOVA test to calculate the F-value and p-value of the test.
/// The F-value is calculated as the ratio of the between-group variance to the within-group variance, and the p-value is calculated using the F-distribution.
///
/// Interpret the results of the ANOVA test. If the p-value is less than the alpha level (usually 0.05), you can conclude that there are significant differences between the means of the groups.
use float_extras::f64::erf;
use itertools::Itertools;
use polars::prelude::{ChunkAgg, ChunkedArray, Float64Type};
use polars::series::Series;
use statrs::distribution::{Continuous, FisherSnedecor};
use statrs::{self};

#[derive(Debug)]
pub struct AnovaResult {
    f_value:                    f64,
    p_value:                    f64,
    degrees_of_freedom_between: usize,
    degrees_of_freedom_within:  usize,
    degrees_of_freedom_total:   usize,
    mean_square_between:        f64,
    mean_square_within:         f64,
    test_normality:             Vec<AndersonDarlingResult>,
    test_equal_variance:        LeveneResult,
}

#[derive(Debug)]
pub struct AndersonDarlingResult {
    statistic: f64,
    p_value:   f64,
}

#[derive(Debug, PartialEq)]
pub struct LeveneResult {
    df1:     f64,
    df2:     f64,
    w_value: f64,
    p_value: f64,
}

pub fn anova_test(data: &[&[f64]]) -> Result<AnovaResult> {
    let total_amount_of_samples = data.iter().map(|group| group.len()).sum::<usize>();
    let amount_of_groups = data.len();

    let mut sum_of_squares_total = 0.0;
    let mut sum_of_squares_between = 0.0;
    let mut sum_of_squares_within = 0.0;

    let mut means = Vec::with_capacity(amount_of_groups);
    for group in data {
        let mean = group.iter().sum::<f64>() / group.len() as f64;
        means.push(mean);

        sum_of_squares_within += group.iter().map(|x| (x - mean).powi(2)).sum::<f64>();
        sum_of_squares_total += group.iter().map(|x| (x - mean).powi(2)).sum::<f64>();
    }

    let grand_mean = means.iter().sum::<f64>() / amount_of_groups as f64;
    sum_of_squares_total -= sum_of_squares_within;
    sum_of_squares_between = sum_of_squares_total - sum_of_squares_within;

    let degrees_of_freedom_between = amount_of_groups - 1;
    let degrees_of_freedom_within = total_amount_of_samples - amount_of_groups;
    let degrees_of_freedom_total = total_amount_of_samples - 1;

    let mean_square_between = sum_of_squares_between / degrees_of_freedom_between as f64;
    let mean_square_within = sum_of_squares_within / degrees_of_freedom_within as f64;

    let f_value = mean_square_between / mean_square_within;

    let p_value =
        FisherSnedecor::new(degrees_of_freedom_between as f64, degrees_of_freedom_within as f64)?.pdf(f_value);

    let test_normality = data.iter().map(|group| anderson_darling_test(group)).collect();
    let test_equal_variance = levene_test(data)?;

    Ok(AnovaResult {
        f_value,
        p_value,
        degrees_of_freedom_between,
        degrees_of_freedom_within,
        degrees_of_freedom_total,
        mean_square_between,
        mean_square_within,
        test_normality,
        test_equal_variance,
    })
}

fn anderson_darling_test(data: &[f64]) -> AndersonDarlingResult {
    let n = data.len() as f64;

    // Sort the data in ascending order
    let mut data = data.to_vec();
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Calculate the cumulative distribution function of the data
    let cdf = data
        .iter()
        .enumerate()
        .map(|(i, x)| ((i + 1) as f64) / (n + 1.0))
        .collect::<Vec<_>>();

    // Calculate the expected cumulative distribution function of the normal distribution
    let mu = data.iter().sum::<f64>() / n;
    let sigma = (data.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    let expected_cdf = data
        .iter()
        .map(|x| (x - mu) / sigma)
        .map(|x| (1.0 + erf(x)) / 2.0)
        .collect::<Vec<_>>();

    // Calculate the test statistic
    let statistic = (1..=data.len())
        .map(|i| {
            let z = (cdf[i - 1] - expected_cdf[i - 1]).powi(2) / expected_cdf[i - 1]
                + (cdf[i] - expected_cdf[i]).powi(2) / (1.0 - expected_cdf[i]);
            (n - (i as f64)) * z
        })
        .sum::<f64>();

    // Calculate the p-value
    let p_value = 2.0 * (-statistic).exp();

    // Return the test result
    AndersonDarlingResult { statistic, p_value }
}

#[allow(non_snake_case)]
fn levene_test(groups: &[&[f64]]) -> Result<LeveneResult> {
    let N = groups.iter().map(|g| g.len()).sum::<usize>();
    let k = groups.len();
    let Z__ = groups.iter().map(|g| g.iter().sum::<f64>()).sum::<f64>() / N as f64;

    let group_mean = groups
        .iter()
        .map(|a| a.iter().sum::<f64>() / a.len() as f64)
        .collect_vec();

    let ssq = groups
        .iter()
        .zip(group_mean.iter())
        .map(|(group, g_mean)| (*group).iter().map(|x| (x - g_mean).abs()).sum::<f64>()
    )
        .collect_vec();

    let total_variance = group_mean.iter().map(|g| (g - Z__).powi(2)).sum::<f64>();
    let variance_within_group = ssq.iter().sum::<f64>();
    let w_value = ((N - k) as f64 / (k - 1) as f64) * (total_variance / variance_within_group);

    let df1 = k as f64 - 1.0;
    let df2 = (N - k) as f64;
    let p_value = FisherSnedecor::new(df1, df2)?.pdf(w_value);
    Ok(LeveneResult {
        w_value,
        p_value,
        df1,
        df2,
    })
}

#[cfg(test)]
mod test_of_tests {
    use super::*;

    #[test]
    fn test_levene() {
        let g1: Vec<f64> = vec![21.0, 23.0, 17.0, 11.0, 9.0, 27.0, 22.0, 12.0, 20.0, 4.0];
        let g2: Vec<f64> = vec![17.0, 16.0, 23.0, 7.0, 26.0, 9.0, 25.0, 21.0, 14.0, 20.0];
        let g3: Vec<f64> = vec![18.0, 22.0, 19.0, 26.0, 13.0, 24.0, 23.0, 17.0, 21.0, 15.0];

        let groups: Vec<&[f64]> = vec![&g1, &g2, &g3];
        let levene = levene_test(&groups).unwrap();
        dbg!(&levene);
        assert_eq!(
            levene,
            LeveneResult {
                df1:     2.0,
                df2:     27.0,
                w_value: 2.016,
                p_value: 0.153,
            }
        )
    }
}
