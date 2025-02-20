/// Determines if a pixel belongs to Mandlebrot's set and returns the path of the sequence.
///
/// # Arguments
///
/// - `x` - The x-coordinate of the pixel.
/// - `y` - The y-coordinate of the pixel.
/// - `width` - The width of the image.
/// - `height` - The height of the image.
/// - `max_iterations` - The maximum number of iterations to check for divergence.
///
/// # Returns
///
/// A tuple containing:
/// - A boolean indicating whether the pixel belongs to Mandlebrot's set.
/// - A vector of `(usize, usize)` tuples representing the x, y coordinates of the pixel at each
///     and every iteration. This is useful for visualizing the path of the sequence.
pub fn is_in_mandelbrot(
    x: f64,
    y: f64,
    width: usize,
    height: usize,
    x_range: (f64, f64),
    y_range: (f64, f64),

    max_iterations: usize,
) -> (Option<usize>, Vec<(usize, usize)>) {
    // Compute the real and imaginary parts of the number c associated with the pixel
    let c_real = map(x, (0.0, width as f64), x_range);
    let c_imaginary = map(y, (0.0, height as f64), y_range);
    let mut pixels = Vec::with_capacity(max_iterations);

    // Initialize the first number in the sequence
    let mut real = 0.0;
    let mut imaginary = 0.0;
    for i in 0..max_iterations {
        // Compute next number in the sequence
        let (new_real, new_imaginary) = calculate_next(c_real, c_imaginary, real, imaginary);

        // Update the current number in the sequence
        real = new_real;
        imaginary = new_imaginary;

        // The sequence diverges to infinity if the modulus of the number is greater than 2
        // Else, we cannot conclude that the sequence diverges
        let diverges = calculate_modulus(real, imaginary) > 2.0;
        if diverges {
            return (Some(i), pixels);
        }
        // Store the x,y coordinates at each iteration
        let i = map_inverse(real, (0.0, width as f64), x_range);
        let j = map_inverse(imaginary, (0.0, height as f64), y_range);
        if i < width as i32 && i >= 0 && j < height as i32 && j >= 0 {
            pixels.push((i as usize, j as usize));
        }
    }
    // We cannot conclude that the sequence diverges so the pixel belongs to Mandlebrot's set
    (None, pixels)
}

/// Takes a number and maps it from one range to another.
///
/// # Arguments
///
/// - `x` - The number to map.
/// - `x_range` - The range of x.
/// - `c_range` - The range to map x to.
///
/// # Returns
///
/// - The mapped number.
fn map(x: f64, x_range: (f64, f64), c_range: (f64, f64)) -> f64 {
    (x - x_range.0) / (x_range.1 - x_range.0) * (c_range.1 - c_range.0) + c_range.0
}

/// Performs an inverse map
///
/// # Arguments
///
/// - `c` - The number to map.
/// - `x_range` - The range of x.
/// - `c_range` - The range to map x to.
///
/// # Returns
///
/// - The mapped number.
fn map_inverse(c: f64, x_range: (f64, f64), c_range: (f64, f64)) -> i32 {
    ((c - c_range.0) / (c_range.1 - c_range.0) * (x_range.1 - x_range.0) + x_range.0) as i32
}

/// Takes the x and y ranges and zooms in by a factor of `zoom_factor`
///
/// # Arguments
///
/// - `x_range` - The range of x.
/// - `y_range` - The range of y.
/// - `zoom_factor` - The factor to zoom in by.
///
/// # Returns
///
/// - The new x and y ranges after zooming in.
pub fn zoom(
    x_range: (f64, f64),
    y_range: (f64, f64),
    zoom_factor: f64,
) -> ((f64, f64), (f64, f64)) {
    // Calculate the center of the current image range
    let x_center = (x_range.0 + x_range.1) / 2.0;
    let y_center = (y_range.0 + y_range.1) / 2.0;

    // Move the range so that the center aligns with the origin
    let x_range_translated = shift(x_range, -x_center);
    let y_range_translated = shift(y_range, -y_center);

    // Scale the range
    let x_range_scaled = scale(x_range_translated, zoom_factor);
    let y_range_scaled = scale(y_range_translated, zoom_factor);

    // Move the range back so that the center returns to its original position
    let x_range_final = shift(x_range_scaled, x_center);
    let y_range_final = shift(y_range_scaled, y_center);

    (x_range_final, y_range_final)
}

/// Takes a range and scales it by a factor
///
/// # Arguments
///
/// - `range` - The range to scale.
/// - `factor` - The factor to scale by.
///
/// # Returns
///
/// - The new range after scaling.
pub fn scale(range: (f64, f64), factor: f64) -> (f64, f64) {
    (range.0 * factor, range.1 * factor)
}

/// Takes a range and shifts it by an offset
///
/// # Arguments
///
/// - `range` - The range to shift.
/// - `offset` - The offset to shift by.
///
/// # Returns
///
/// - The new range after shifting.
pub fn shift(range: (f64, f64), offset: f64) -> (f64, f64) {
    (range.0 + offset, range.1 + offset)
}

/// Takes a range and returns the shift speed
///
/// # Arguments
///
/// - `range` - The range to calculate the shift speed for.
/// - `factor` - How big the shift should be.
///
/// # Returns
///
/// - The shift speed.
pub fn get_shift_speed(range: (f64, f64), factor: u32) -> f64 {
    (range.1 - range.0) / factor as f64
}

/// Takes the real and imaginary parts of a number as arguments. Returns the modulus of the number
///
/// # Arguments
///
/// - `real_part` - The real part of the number.
/// - `imaginary_part` - The imaginary part of the number.
///
/// # Returns
///
/// - The modulus of the number.
fn calculate_modulus(real_part: f64, imaginary_part: f64) -> f64 {
    (real_part.powi(2) + imaginary_part.powi(2)).sqrt()
}

/// Calculates the next number in the sequence
///
/// # Arguments
///
/// - `c_real`: The real part of the number c.
/// - `c_imaginary`: The imaginary part of the number c.
/// - `real`: The real part of the number z[n].
/// - `imaginary`: The imaginary part of the number z[n].
///
/// # Returns
///
/// - A tuple containing the real and imaginary parts of the next number in the sequence.
fn calculate_next(c_real: f64, c_imaginary: f64, real: f64, imaginary: f64) -> (f64, f64) {
    let res_real = real.powi(2) - imaginary.powi(2) + c_real;
    let res_imaginary = 2.0 * real * imaginary + c_imaginary;
    (res_real, res_imaginary)
}
