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
    max_iterations: usize,
) -> (bool, Vec<(usize, usize)>) {
    // Compute the real and imaginary parts of the number c associated with the pixel
    let c_real = find_real_from_x(x, width);
    let c_imaginary = find_imaginary_from_y(y, height);
    let mut pixels = Vec::with_capacity(max_iterations);

    // Initialize the first number in the sequence
    let mut real = 0.0;
    let mut imaginary = 0.0;
    for _ in 0..max_iterations {
        // Compute next number in the sequence
        let new_real = calculate_next_real(c_real, real, imaginary);
        let new_imaginary = calculate_next_imaginary(c_imaginary, real, imaginary);

        // Update the current number in the sequence
        real = new_real;
        imaginary = new_imaginary;

        // The sequence diverges to infinity if the modulus of the number is greater than 2
        // Else, we cannot conclude that the sequence diverges
        let diverges = calculate_modulus(real, imaginary) > 2.0;
        if diverges {
            return (false, pixels);
        }
        // Store the x,y coordinates at each iteration
        let i = find_x_from_real(real, width);
        let j = find_y_from_imaginary(imaginary, height);
        if i < width as i32 && i >= 0 && j < height as i32 && j >= 0 {
            pixels.push((i as usize, j as usize));
        }
    }
    // We cannot conclude that the sequence diverges so the pixel belongs to Mandlebrot's set
    (true, pixels)
}

/// Takes a pixel as an argument. Returns the real part of the number c
///
/// # Arguments
///
/// - `x` - The x-coordinate of the pixel.
/// - `width` - The width of the image.
///
/// # Returns
///
/// - The real part of the number c.
fn find_real_from_x(x: f64, width: usize) -> f64 {
    2.0 * (x - width as f64 / 1.35) / (width as f64 - 1.0)
}

/// Takes a pixel as an argument. Returns the imaginary part of the number c
///
/// # Arguments
///
/// - `y` - The y-coordinate of the pixel.
/// - `height` - The height of the image.
///
/// # Returns
///
/// - The imaginary part of the number c.
fn find_imaginary_from_y(y: f64, height: usize) -> f64 {
    2.0 * (y - height as f64 / 2.0) / (height as f64 - 1.0)
}

/// Takes a real number as an argument. Returns the pixel associated with this real number
///
/// # Arguments
///
/// - `real` - The real part of the number.
/// - `width` - The width of the image.
///
/// # Returns
///
/// - The x-coordinate of the pixel.
fn find_x_from_real(real: f64, width: usize) -> i32 {
    (real * (width as f64 - 1.0) / 2.0 + width as f64 / 1.35) as i32
}

/// Takes an imaginary number as an argument. Returns the pixel associated with this imaginary number
///
/// # Arguments
///
/// - `imaginary` - The imaginary part of the number.
/// - `height` - The height of the image.
///
/// # Returns
///
/// - The y-coordinate of the pixel.
fn find_y_from_imaginary(imaginary: f64, height: usize) -> i32 {
    (imaginary * (height as f64 - 1.0) / 2.0 + height as f64 / 2.0) as i32
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

/// Calculates the next real number in the sequence
///
/// z[n+1] = z[n]^2 + c  =>  x[n+1] = x[n]^2 - y[n]^2 + Re(c)
/// x represent the real part of x_n
/// y represents the imaginary part of x_n
///
/// # Arguments
///
/// - `c_imaginary` - The imaginary part of the number c.
/// - `real` - The real part of the number z[n].
/// - `imaginary` - The imaginary part of the number z[n].
///
/// # Returns
///
/// - The next real number in the sequence.
fn calculate_next_real(c_real: f64, real: f64, imaginary: f64) -> f64 {
    real.powi(2) - imaginary.powi(2) + c_real
}

/// Calculates the next imaginary number in the sequence
///
/// z[n+1] = z[n]^2 + c  =>  y[n+1] = 2 * x[n] * y[n] + Im(c)
/// x represent the real part of x_n
/// y represents the imaginary part of x_n
///
/// # Arguments
///
/// - `c_imaginary` - The imaginary part of the number c.
/// - `real` - The real part of the number z[n].
/// - `imaginary` - The imaginary part of the number z[n].
///
/// # Returns
///
/// - The next imaginary number in the sequence.
fn calculate_next_imaginary(c_imaginary: f64, real: f64, imaginary: f64) -> f64 {
    2.0 * real * imaginary + c_imaginary
}
