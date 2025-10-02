/**
 * Numerical Integration Utilities
 *
 * Provides methods for numerical integration using different algorithms.
 * These utilities are used across various DSP tools for calculating
 * Fourier coefficients, transforms, and other integrals.
 */

/**
 * Simpson's Rule for numerical integration
 *
 * Most accurate for smooth functions, uses parabolic approximations.
 * Requires an even number of steps for proper implementation.
 *
 * @param fn - Function to integrate
 * @param a - Lower bound
 * @param b - Upper bound
 * @param steps - Number of integration steps (will be made even if odd)
 * @returns Numerical approximation of the integral
 */
export function integrateNumerically(
    fn: (t: number) => number,
    a: number,
    b: number,
    steps: number = 1000
): number {
    if (a === b) return 0;
    if (steps % 2 !== 0) steps++; // Steps must be even for Simpson's rule

    const h = (b - a) / steps;
    let sum = fn(a) + fn(b);

    for (let i = 1; i < steps; i++) {
        const x = a + i * h;
        sum += (i % 2 !== 0) ? 4 * fn(x) : 2 * fn(x);
    }

    return (h / 3) * sum;
}

/**
 * Trapezoidal Rule for numerical integration
 *
 * Simpler but less accurate than Simpson's rule.
 * Good for quick approximations or when function is linear.
 *
 * @param fn - Function to integrate
 * @param a - Lower bound
 * @param b - Upper bound
 * @param steps - Number of integration steps
 * @returns Numerical approximation of the integral
 */
export function integrateTrapezoid(
    fn: (t: number) => number,
    a: number,
    b: number,
    steps: number = 1000
): number {
    if (a === b) return 0;

    const h = (b - a) / steps;
    let sum = (fn(a) + fn(b)) / 2;

    for (let i = 1; i < steps; i++) {
        sum += fn(a + i * h);
    }

    return h * sum;
}

/**
 * Adaptive Simpson's Rule
 *
 * Automatically adjusts step size based on error estimation.
 * Better for functions with varying complexity.
 *
 * @param fn - Function to integrate
 * @param a - Lower bound
 * @param b - Upper bound
 * @param tolerance - Error tolerance
 * @returns Numerical approximation of the integral
 */
export function integrateAdaptive(
    fn: (t: number) => number,
    a: number,
    b: number,
    tolerance: number = 1e-6
): number {
    const adaptiveStep = (left: number, right: number, tol: number): number => {
        const mid = (left + right) / 2;
        const h = (right - left) / 6;

        const fLeft = fn(left);
        const fMid = fn(mid);
        const fRight = fn(right);

        const wholeSimpson = h * (fLeft + 4 * fMid + fRight);

        const leftMid = (left + mid) / 2;
        const rightMid = (mid + right) / 2;
        const leftSimpson = (h / 2) * (fLeft + 4 * fn(leftMid) + fMid);
        const rightSimpson = (h / 2) * (fMid + 4 * fn(rightMid) + fRight);
        const combinedSimpson = leftSimpson + rightSimpson;

        const error = Math.abs(combinedSimpson - wholeSimpson) / 15;

        if (error < tol) {
            return combinedSimpson + error;
        } else {
            return adaptiveStep(left, mid, tol / 2) + adaptiveStep(mid, right, tol / 2);
        }
    };

    if (a === b) return 0;
    return adaptiveStep(a, b, tolerance);
}