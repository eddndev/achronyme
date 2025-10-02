/**
 * Fourier Transform Calculations
 *
 * Implements Discrete Fourier Transform (DFT) for piecewise functions
 * using numerical integration (Simpson's Rule).
 */

import * as math from 'mathjs';
import { integrateNumerically } from '../utils/numerical-integration';

// --- Interfaces ---

/**
 * Complex number representation
 */
export interface ComplexNumber {
    real: number;
    imag: number;
}

/**
 * Compiled piecewise function segment
 */
export interface CompiledFunction {
    compiled: math.EvalFunction;
    domainStart: number;
    domainEnd: number;
}

/**
 * Result of Fourier Transform calculation
 */
export interface FourierTransformResult {
    success: boolean;
    omega: number[];       // Angular frequency array (rad/s)
    magnitude: number[];   // |F(ω)|
    phase: number[];       // ∠F(ω) in radians
    real: number[];        // Real part of F(ω)
    imag: number[];        // Imaginary part of F(ω)
    error?: string;
}

/**
 * Time domain representation for plotting
 */
export interface TimeDomainData {
    t: number[];
    values: number[];
}

// --- Main Fourier Transform Calculation ---

/**
 * Calculate Continuous Fourier Transform using DFT with numerical integration
 *
 * Formula: F(ω) = ∫_{-∞}^{∞} f(t) e^{-jωt} dt
 *
 * Split into real and imaginary parts:
 * F(ω) = ∫ f(t) cos(ωt) dt - j ∫ f(t) sin(ωt) dt
 *
 * @param functions - Array of compiled piecewise functions
 * @param omegaRange - Maximum angular frequency to compute (rad/s)
 * @param resolution - Number of frequency points to calculate
 * @returns Fourier Transform result with magnitude and phase
 */
export function calculateFourierTransform(
    functions: CompiledFunction[],
    omegaRange: number,
    resolution: number
): FourierTransformResult {
    try {
        if (!functions || functions.length === 0) {
            throw new Error("No se proporcionaron funciones para el cálculo.");
        }

        // Validate omega range and resolution
        if (omegaRange <= 0) {
            throw new Error("El rango de frecuencia debe ser positivo.");
        }

        if (resolution < 10) {
            throw new Error("La resolución debe ser al menos 10 puntos.");
        }

        const omega: number[] = [];
        const magnitude: number[] = [];
        const phase: number[] = [];
        const real: number[] = [];
        const imag: number[] = [];

        // Frequency step size
        const omegaStep = (2 * omegaRange) / resolution;

        // Calculate F(ω) for each frequency
        for (let w = -omegaRange; w <= omegaRange; w += omegaStep) {
            omega.push(w);

            // Initialize complex number F(ω) = realPart + j * imagPart
            let realPart = 0;
            let imagPart = 0;

            // Integrate over each piecewise segment
            for (const func of functions) {
                try {
                    // Real part: ∫ f(t) * cos(ωt) dt
                    const cosIntegrand = (t: number) => {
                        const ft = func.compiled.evaluate({ t });
                        return ft * Math.cos(w * t);
                    };
                    realPart += integrateNumerically(
                        cosIntegrand,
                        func.domainStart,
                        func.domainEnd,
                        1000 // Integration steps for accuracy
                    );

                    // Imaginary part: -∫ f(t) * sin(ωt) dt (negative for e^{-jωt})
                    const sinIntegrand = (t: number) => {
                        const ft = func.compiled.evaluate({ t });
                        return ft * Math.sin(w * t);
                    };
                    imagPart -= integrateNumerically(
                        sinIntegrand,
                        func.domainStart,
                        func.domainEnd,
                        1000
                    );
                } catch (e: any) {
                    console.warn(`Error evaluating function at ω=${w}:`, e.message);
                    // Continue with next segment
                }
            }

            // Store complex components
            real.push(realPart);
            imag.push(imagPart);

            // Magnitude: |F(ω)| = √(Real² + Imag²)
            const mag = Math.sqrt(realPart * realPart + imagPart * imagPart);
            magnitude.push(mag);

            // Phase: ∠F(ω) = atan2(Imag, Real)
            // Note: Returns phase in radians from -π to π
            const ph = Math.atan2(imagPart, realPart);
            phase.push(ph);
        }

        console.log(`[Fourier Transform] Calculated ${omega.length} frequency points`);
        console.log(`[Fourier Transform] ω range: [${omega[0].toFixed(2)}, ${omega[omega.length-1].toFixed(2)}] rad/s`);

        return {
            success: true,
            omega,
            magnitude,
            phase,
            real,
            imag
        };

    } catch (error: any) {
        console.error('[Fourier Transform] Calculation error:', error);
        return {
            success: false,
            omega: [],
            magnitude: [],
            phase: [],
            real: [],
            imag: [],
            error: error.message
        };
    }
}

/**
 * Generate time domain samples for plotting
 *
 * Evaluates piecewise function at uniform time points
 * across the entire domain.
 *
 * @param functions - Array of compiled piecewise functions
 * @param numPoints - Number of points to sample
 * @returns Time domain data for plotting
 */
export function generateTimeDomainData(
    functions: CompiledFunction[],
    numPoints: number = 500
): TimeDomainData {
    if (!functions || functions.length === 0) {
        return { t: [], values: [] };
    }

    const t: number[] = [];
    const values: number[] = [];

    // Determine overall time range
    const tStart = functions[0].domainStart;
    const tEnd = functions[functions.length - 1].domainEnd;
    const tStep = (tEnd - tStart) / numPoints;

    // Sample function at uniform intervals
    for (let time = tStart; time <= tEnd; time += tStep) {
        t.push(time);

        // Find which segment contains this time point
        let value = 0;
        for (const func of functions) {
            if (time >= func.domainStart - 1e-9 && time <= func.domainEnd + 1e-9) {
                try {
                    value = func.compiled.evaluate({ t: time });
                    if (!isFinite(value)) {
                        value = 0;
                    }
                } catch (e) {
                    value = 0;
                }
                break;
            }
        }

        values.push(value);
    }

    return { t, values };
}

/**
 * Calculate energy spectral density
 *
 * ESD(ω) = |F(ω)|²
 *
 * @param magnitude - Magnitude spectrum |F(ω)|
 * @returns Energy spectral density
 */
export function calculateEnergySpectralDensity(magnitude: number[]): number[] {
    return magnitude.map(mag => mag * mag);
}

/**
 * Calculate total signal energy (Parseval's Theorem)
 *
 * E = ∫ |f(t)|² dt = (1/2π) ∫ |F(ω)|² dω
 *
 * @param functions - Compiled piecewise functions
 * @returns Total energy of the signal
 */
export function calculateTotalEnergy(functions: CompiledFunction[]): number {
    let energy = 0;

    for (const func of functions) {
        const energyIntegrand = (t: number) => {
            const ft = func.compiled.evaluate({ t });
            return ft * ft;
        };

        energy += integrateNumerically(
            energyIntegrand,
            func.domainStart,
            func.domainEnd,
            1000
        );
    }

    return energy;
}

/**
 * Unwrap phase to avoid discontinuities
 *
 * Converts phase from [-π, π] range to continuous values
 * by adding/subtracting 2π when jumps are detected.
 *
 * @param phase - Array of phase values in radians
 * @returns Unwrapped phase array
 */
export function unwrapPhase(phase: number[]): number[] {
    if (phase.length === 0) return [];

    const unwrapped: number[] = [phase[0]];
    let offset = 0;

    for (let i = 1; i < phase.length; i++) {
        let delta = phase[i] - phase[i - 1];

        // Detect jump greater than π
        if (delta > Math.PI) {
            offset -= 2 * Math.PI;
        } else if (delta < -Math.PI) {
            offset += 2 * Math.PI;
        }

        unwrapped.push(phase[i] + offset);
    }

    return unwrapped;
}