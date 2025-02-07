/// Recalibrates the pixel values of a 2D array representing an image to the range [0, 255].
///
/// Taken from my teacher, Max Mignotte:
/// https://www.iro.umontreal.ca/~mignotte/
///
/// # Arguments
///
/// - `mat` - A mutable reference to a 2D vector (`Vec<Vec<f64>>`) containing pixel values.
///     The pixel values should be of type `f64`, but they may not necessarily be within the
///     [0, 255] range before recalibration.
///
/// # Panics
///
/// This function will panic if the 2D array `mat` is empty (i.e., has no rows or columns) or if
/// any row has a different length from the first row.
pub fn recalibrate(mat: &mut [Vec<f64>]) {
    let width = mat[0].len();
    let height = mat.len();

    // Find the min luma value
    let mut luma_min = mat[0][0];
    (0..height).for_each(|i| {
        (0..width).for_each(|j| {
            luma_min = mat[i][j].min(luma_min);
        });
    });

    // Subtract min from all pixels in the image
    (0..height).for_each(|i| {
        (0..width).for_each(|j| {
            mat[i][j] -= luma_min;
        });
    });

    // Find the max luma value
    let mut luma_max = mat[0][0];
    (0..height).for_each(|i| {
        (0..width).for_each(|j| {
            luma_max = mat[i][j].max(luma_max);
        });
    });

    // Recalibrate the image
    (0..height).for_each(|i| {
        (0..width).for_each(|j| {
            mat[i][j] *= 255.0 / luma_max;
        });
    });
}

/// Performs histogram equalization on a 2D array of pixel values (luminance).
///
/// Taken from my teacher, Max Mignotte:
/// https://www.iro.umontreal.ca/~mignotte/
///
/// # Arguments
///
/// - `array` - A mutable 2D vector of `f64` values representing the pixel intensities
///   of an image, typically in the range [0.0, 255.0].
/// - `thresh` - A threshold value that filters out pixel values lower than this value
///   when calculating the histogram. Pixels with values greater than `thresh` are
///   included in the histogram calculation.
///
/// # Description
///
/// The function performs the following steps:
/// 1. **Histogram Calculation**: It calculates a normalized histogram (`histo_ng`) for pixel values
///    greater than `thresh` across all pixels in the 2D array. The frequency of each pixel value
///    is counted and normalized by the total number of pixels greater than the threshold.
/// 2. **Cumulative Distribution Function (CDF)**: It computes the cumulative distribution of the
///    normalized histogram (`FnctRept`).
/// 3. **Scaling**: The CDF is scaled to fit within the range [0, 255].
/// 4. **Equalization**: The pixel values in the original 2D array are updated using the scaled CDF
///    to perform the histogram equalization, improving the contrast of the image.
///
/// # Panics
///
/// This function will panic if `array` is empty (i.e., no rows or columns) or if any row in the
/// 2D array has a different length than the first row.
pub fn equalize(array: &mut [Vec<f64>], thresh: f64) {
    let width = array[0].len();
    let height = array.len();

    // Calculate histogram Ng (normalized)
    let mut histo_ng = vec![0.0; 256];
    let mut n = 0;
    (0..height).for_each(|i| {
        (0..width).for_each(|j| {
            let luma = array[i][j];
            if luma > thresh {
                histo_ng[luma as usize] += 1.0;
                n += 1;
            }
        });
    });

    // Normalize the histogram
    (0..256).for_each(|i| {
        histo_ng[i] /= n as f64;
    });

    // Calculate cumulative distribution function (FnctRept)
    let mut fnct_rept = vec![0.0; 256];
    (1..256).for_each(|i| {
        fnct_rept[i] = fnct_rept[i - 1] + histo_ng[i];
    });

    // Scale the cumulative distribution to the 0-255 range
    (0..256).for_each(|i| {
        fnct_rept[i] = (fnct_rept[i] * 255.0).round();
    });

    // Equalize the image
    (0..height).for_each(|i| {
        (0..width).for_each(|j| {
            array[i][j] = fnct_rept[array[i][j] as usize];
        });
    });
}
